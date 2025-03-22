use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use web_sys::{HtmlInputElement, KeyboardEvent};
use crate::filesystem::{FileSystem, FileType, FileMetadata};
use std::path::Path;

pub struct Terminal {
    fs: Rc<RefCell<FileSystem>>,
    current_directory: String,
    command_history: Vec<String>,
    history_index: Option<usize>,
    output_history: Vec<TerminalOutput>,
    current_input: String,
    input_ref: NodeRef,
}

pub enum TerminalMsg {
    InputChanged(String),
    ExecuteCommand,
    KeyDown(KeyboardEvent),
    ScrollToBottom,
}

#[derive(Properties, Clone, PartialEq)]
pub struct TerminalProps {
    pub fs: Rc<RefCell<FileSystem>>,
}

#[derive(Clone, PartialEq)]
enum TerminalOutput {
    Command(String),
    StandardOutput(String),
    ErrorOutput(String),
}

impl Component for Terminal {
    type Message = TerminalMsg;
    type Properties = TerminalProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            fs: Rc::clone(&ctx.props().fs),
            current_directory: "/home".to_string(),
            command_history: Vec::new(),
            history_index: None,
            output_history: vec![
                TerminalOutput::StandardOutput("WasmOS Terminal v0.1.0".to_string()),
                TerminalOutput::StandardOutput("Type 'help' for available commands.".to_string()),
            ],
            current_input: String::new(),
            input_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TerminalMsg::InputChanged(value) => {
                self.current_input = value;
                true
            }
            TerminalMsg::ExecuteCommand => {
                let command = self.current_input.trim().to_string();
                if !command.is_empty() {
                    self.execute_command(&command);
                    self.current_input = String::new();
                    ctx.link().send_message(TerminalMsg::ScrollToBottom);
                }
                true
            }
            TerminalMsg::KeyDown(event) => {
                match event.key().as_str() {
                    "Enter" => {
                        ctx.link().send_message(TerminalMsg::ExecuteCommand);
                    }
                    "ArrowUp" => {
                        event.prevent_default();
                        // Navigate command history (previous)
                        if !self.command_history.is_empty() {
                            let index = match self.history_index {
                                None => self.command_history.len() - 1,
                                Some(i) if i > 0 => i - 1,
                                Some(i) => i,
                            };
                            self.history_index = Some(index);
                            self.current_input = self.command_history[index].clone();
                        }
                        return true;
                    }
                    "ArrowDown" => {
                        event.prevent_default();
                        // Navigate command history (next)
                        match self.history_index {
                            Some(i) if i < self.command_history.len() - 1 => {
                                let next_index = i + 1;
                                self.history_index = Some(next_index);
                                self.current_input = self.command_history[next_index].clone();
                            }
                            Some(_) => {
                                self.history_index = None;
                                self.current_input = String::new();
                            }
                            None => {}
                        }
                        return true;
                    }
                    "Tab" => {
                        event.prevent_default();
                        // Implement tab completion
                        let input = self.current_input.trim();
                        if !input.is_empty() {
                            let parts: Vec<&str> = input.split_whitespace().collect();
                            
                            if parts.len() == 1 || (parts.len() > 1 && !parts[0].is_empty()) {
                                // Command completion
                                if parts.len() == 1 {
                                    let cmd = parts[0];
                                    let commands = vec!["help", "cd", "pwd", "ls", "cat", "echo", "clear", "mkdir", "touch", "rm", "history"];
                                    let matches: Vec<&str> = commands.into_iter()
                                        .filter(|c| c.starts_with(cmd))
                                        .collect();
                                    
                                    if matches.len() == 1 {
                                        // Single match, complete it
                                        self.current_input = matches[0].to_string();
                                        return true;
                                    } else if matches.len() > 1 {
                                        // Multiple matches, show options
                                        self.output_history.push(TerminalOutput::Command(format!("{} $ {}", self.current_directory, input)));
                                        self.output_history.push(TerminalOutput::StandardOutput(
                                            matches.join("  ")
                                        ));
                                        return true;
                                    }
                                }
                                
                                // File/directory completion
                                if parts.len() > 1 || parts[0] == "cd" || parts[0] == "ls" || parts[0] == "cat" || parts[0] == "rm" || parts[0] == "touch" {
                                    let path_part = if parts.len() > 1 { parts[parts.len() - 1] } else { "" };
                                    let path_to_complete = self.resolve_path(path_part);
                                    
                                    // Get directory part and file prefix
                                    let (dir_path, file_prefix) = if path_to_complete.ends_with('/') {
                                        (path_to_complete.clone(), "".to_string())
                                    } else {
                                        let path = Path::new(&path_to_complete);
                                        match path.parent() {
                                            Some(parent) => (parent.to_string_lossy().to_string(), 
                                                             path.file_name()
                                                                 .map(|f| f.to_string_lossy().to_string())
                                                                 .unwrap_or_default()),
                                            None => ("/".to_string(), path_to_complete.clone())
                                        }
                                    };
                                    
                                    // List files in directory
                                    match self.fs.borrow().list_directory(&dir_path) {
                                        Ok(files) => {
                                            // Filter files that match the prefix
                                            let matches: Vec<FileMetadata> = files.into_iter()
                                                .filter(|f| f.name.starts_with(&file_prefix))
                                                .collect();
                                            
                                            if matches.len() == 1 {
                                                // Single match, complete it
                                                let completed_path = if path_part.starts_with('/') {
                                                    if dir_path == "/" {
                                                        format!("/{}", matches[0].name)
                                                    } else {
                                                        format!("{}/{}", dir_path, matches[0].name)
                                                    }
                                                } else {
                                                    matches[0].name.clone()
                                                };
                                                
                                                // Add trailing slash for directories
                                                let completed_path = if matches[0].file_type == FileType::Directory && !completed_path.ends_with('/') {
                                                    format!("{}/", completed_path)
                                                } else {
                                                    completed_path
                                                };
                                                
                                                // Replace the path part in the command
                                                if parts.len() > 1 {
                                                    let mut new_parts = parts[0..parts.len()-1].to_vec();
                                                    new_parts.push(&completed_path);
                                                    self.current_input = new_parts.join(" ");
                                                } else {
                                                    self.current_input = format!("{} {}", parts[0], completed_path);
                                                }
                                                
                                                return true;
                                            } else if matches.len() > 1 {
                                                // Multiple matches, show options
                                                self.output_history.push(TerminalOutput::Command(format!("{} $ {}", self.current_directory, input)));
                                                let matches_str = matches.iter()
                                                    .map(|f| {
                                                        match f.file_type {
                                                            FileType::Directory => format!("{}/", f.name),
                                                            FileType::File => f.name.clone(),
                                                        }
                                                    })
                                                    .collect::<Vec<String>>()
                                                    .join("  ");
                                                self.output_history.push(TerminalOutput::StandardOutput(matches_str));
                                                
                                                return true;
                                            }
                                        },
                                        Err(_) => {
                                            // Couldn't read directory, do nothing
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
                false
            }
            TerminalMsg::ScrollToBottom => {
                // This happens after rendering
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onkeydown = ctx.link().callback(TerminalMsg::KeyDown);
        let oninput = ctx.link().callback(|e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            TerminalMsg::InputChanged(input.value())
        });
        
        html! {
            <div class="terminal" style="height: 100%; overflow: hidden; display: flex; flex-direction: column; background-color: #1e1e1e; color: #f0f0f0; font-family: monospace;">
                <div class="terminal-output" style="flex-grow: 1; overflow-y: auto; padding: 8px; white-space: pre-wrap;">
                    {
                        self.output_history.iter().map(|output| {
                            match output {
                                TerminalOutput::Command(text) => {
                                    html! { <div style="color: #f0f0f0; padding: 2px 0;">{ text }</div> }
                                }
                                TerminalOutput::StandardOutput(text) => {
                                    html! { <div style="color: #a0a0a0; padding: 2px 0;">{ text }</div> }
                                }
                                TerminalOutput::ErrorOutput(text) => {
                                    html! { <div style="color: #ff6b6b; padding: 2px 0;">{ text }</div> }
                                }
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="terminal-input" style="display: flex; padding: 8px; border-top: 1px solid #333;">
                    <span>{ format!("{} $ ", self.current_directory) }</span>
                    <input 
                        type="text"
                        style="flex-grow: 1; background-color: transparent; border: none; color: #f0f0f0; font-family: monospace; outline: none;"
                        value={self.current_input.clone()}
                        ref={self.input_ref.clone()}
                        {oninput}
                        {onkeydown}
                        autocomplete="off"
                        spellcheck="false"
                    />
                </div>
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            // Focus input on first render
            if let Some(input) = self.input_ref.cast::<HtmlInputElement>() {
                let _ = input.focus();
            }
        }
        
        // Scroll to bottom when new output is added
        if let Some(output_div) = web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.query_selector(".terminal-output").ok())
            .flatten()
        {
            output_div.set_scroll_top(output_div.scroll_height());
        }
    }
}

impl Terminal {
    fn execute_command(&mut self, command: &str) {
        self.output_history.push(TerminalOutput::Command(format!("{} $ {}", self.current_directory, command)));
        
        // Save command to history
        if !command.trim().is_empty() && (!self.command_history.is_empty() && self.command_history.last().unwrap() != command) {
            self.command_history.push(command.to_string());
        }
        
        if self.command_history.len() > 50 {
            self.command_history.remove(0);
        }
        
        self.history_index = None;
        
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "help" => {
                self.output_history.push(TerminalOutput::StandardOutput(
                    "Available commands:\n\
                    help       - Show this help\n\
                    cd [path]  - Change directory\n\
                    pwd        - Print working directory\n\
                    ls         - List directory contents\n\
                    cat [file] - Display file contents\n\
                    echo [text]- Display text\n\
                    clear      - Clear terminal\n\
                    mkdir [dir]- Create directory\n\
                    touch [file]- Create empty file\n\
                    rm [path]  - Remove file or directory\n\
                    history    - Display command history".to_string()
                ));
            }
            "cd" => {
                let target = if parts.len() > 1 { parts[1] } else { "/" };
                let path = self.resolve_path(target);
                
                match self.fs.borrow().list_directory(&path) {
                    Ok(_) => {
                        self.current_directory = path;
                    }
                    Err(e) => {
                        self.output_history.push(TerminalOutput::ErrorOutput(format!("cd: {}", e)));
                    }
                }
            }
            "pwd" => {
                self.output_history.push(TerminalOutput::StandardOutput(self.current_directory.clone()));
            }
            "ls" => {
                let path = if parts.len() > 1 {
                    self.resolve_path(parts[1])
                } else {
                    self.current_directory.clone()
                };
                
                match self.fs.borrow().list_directory(&path) {
                    Ok(files) => {
                        let mut output = String::new();
                        for file in files {
                            let type_indicator = match file.file_type {
                                FileType::Directory => "/",
                                FileType::File => "",
                            };
                            output.push_str(&format!("{}{}\n", file.name, type_indicator));
                        }
                        self.output_history.push(TerminalOutput::StandardOutput(output));
                    }
                    Err(e) => {
                        self.output_history.push(TerminalOutput::ErrorOutput(format!("ls: {}", e)));
                    }
                }
            }
            "cat" => {
                if parts.len() < 2 {
                    self.output_history.push(TerminalOutput::ErrorOutput("cat: missing file operand".to_string()));
                    return;
                }
                
                let path = self.resolve_path(parts[1]);
                match self.fs.borrow().read_file(&path) {
                    Ok(content) => {
                        self.output_history.push(TerminalOutput::StandardOutput(content));
                    }
                    Err(e) => {
                        self.output_history.push(TerminalOutput::ErrorOutput(format!("cat: {}", e)));
                    }
                }
            }
            "echo" => {
                let text = if parts.len() > 1 {
                    parts[1..].join(" ")
                } else {
                    String::new()
                };
                self.output_history.push(TerminalOutput::StandardOutput(text));
            }
            "clear" => {
                self.output_history = Vec::new();
            }
            "mkdir" => {
                if parts.len() < 2 {
                    self.output_history.push(TerminalOutput::ErrorOutput("mkdir: missing directory operand".to_string()));
                    return;
                }
                
                let path = self.resolve_path(parts[1]);
                match self.fs.borrow_mut().create_directory(&path, false) {
                    Ok(_) => {},
                    Err(e) => {
                        self.output_history.push(TerminalOutput::ErrorOutput(format!("mkdir: {}", e)));
                    }
                }
            }
            "touch" => {
                if parts.len() < 2 {
                    self.output_history.push(TerminalOutput::ErrorOutput("touch: missing file operand".to_string()));
                    return;
                }
                
                let path = self.resolve_path(parts[1]);
                match self.fs.borrow_mut().write_file(&path, "") {
                    Ok(_) => {},
                    Err(e) => {
                        self.output_history.push(TerminalOutput::ErrorOutput(format!("touch: {}", e)));
                    }
                }
            }
            "rm" => {
                if parts.len() < 2 {
                    self.output_history.push(TerminalOutput::ErrorOutput("rm: missing operand".to_string()));
                    return;
                }
                
                let path = self.resolve_path(parts[1]);
                let recursive = parts.len() > 2 && parts[2] == "-r";
                
                match self.fs.borrow_mut().delete(&path, recursive) {
                    Ok(_) => {},
                    Err(e) => {
                        self.output_history.push(TerminalOutput::ErrorOutput(format!("rm: {}", e)));
                    }
                }
            }
            "history" => {
                // Display command history
                if self.command_history.is_empty() {
                    self.output_history.push(TerminalOutput::StandardOutput("No command history".to_string()));
                } else {
                    let mut history_output = "Command History:".to_string();
                    for (i, cmd) in self.command_history.iter().enumerate() {
                        history_output.push_str(&format!("\n{}: {}", i + 1, cmd));
                    }
                    self.output_history.push(TerminalOutput::StandardOutput(history_output));
                }
            }
            _ => {
                self.output_history.push(TerminalOutput::ErrorOutput(format!("Unknown command: {}", parts[0])));
            }
        }
    }

    fn resolve_path(&self, path: &str) -> String {
        if path.starts_with('/') {
            path.to_string()
        } else {
            let current = if self.current_directory.ends_with('/') {
                self.current_directory.clone()
            } else {
                format!("{}/", self.current_directory)
            };
            
            format!("{}{}", current, path)
        }
    }
}