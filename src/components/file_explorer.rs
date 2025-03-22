use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::filesystem::{FileSystem, FileType, FileMetadata};
use wasm_bindgen::JsValue;

pub struct FileExplorer {
    fs: Rc<RefCell<FileSystem>>,
    current_path: String,
    files: Vec<FileMetadata>,
    selected_file: Option<String>,
    error_message: Option<String>,
}

pub enum FileExplorerMsg {
    NavigateTo(String),
    NavigateUp,
    Refresh,
    SelectFile(String),
    OpenFile(String),
    DeleteFile(String),
    CreateNewFile,
    CreateNewDirectory,
    Error(String),
    ClearError,
}

#[derive(Properties, Clone, PartialEq)]
pub struct FileExplorerProps {
    pub fs: Rc<RefCell<FileSystem>>,
    pub on_open_file: Callback<(String, String)>, // (path, file_type)
}

impl Component for FileExplorer {
    type Message = FileExplorerMsg;
    type Properties = FileExplorerProps;

    fn create(ctx: &Context<Self>) -> Self {
        let fs = Rc::clone(&ctx.props().fs);
        let current_path = "/home".to_string();
        
        // Load initial directory
        let files = match fs.borrow().list_directory(&current_path) {
            Ok(files) => files,
            Err(_) => Vec::new(),
        };

        Self {
            fs,
            current_path,
            files,
            selected_file: None,
            error_message: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FileExplorerMsg::NavigateTo(path) => {
                match self.fs.borrow().list_directory(&path) {
                    Ok(files) => {
                        self.current_path = path;
                        self.files = files;
                        self.selected_file = None;
                        true
                    },
                    Err(e) => {
                        self.error_message = Some(e);
                        true
                    }
                }
            },
            FileExplorerMsg::NavigateUp => {
                let parent = std::path::Path::new(&self.current_path)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "/".to_string());
                
                ctx.link().send_message(FileExplorerMsg::NavigateTo(parent));
                false
            },
            FileExplorerMsg::Refresh => {
                match self.fs.borrow().list_directory(&self.current_path) {
                    Ok(files) => {
                        self.files = files;
                        true
                    },
                    Err(e) => {
                        self.error_message = Some(e);
                        true
                    }
                }
            },
            FileExplorerMsg::SelectFile(path) => {
                self.selected_file = Some(path);
                true
            },
            FileExplorerMsg::OpenFile(name) => {
                let full_path = format!("{}/{}", self.current_path, name);
                
                // Check if it's a directory or file
                for file in &self.files {
                    if file.name == name {
                        match file.file_type {
                            FileType::Directory => {
                                ctx.link().send_message(FileExplorerMsg::NavigateTo(full_path));
                                return false;
                            },
                            FileType::File => {
                                // Notify parent to open file
                                ctx.props().on_open_file.emit((full_path, "text".to_string()));
                                return false;
                            }
                        }
                    }
                }
                false
            },
            FileExplorerMsg::DeleteFile(name) => {
                let full_path = format!("{}/{}", self.current_path, name);
                
                match self.fs.borrow_mut().delete(&full_path, true) {
                    Ok(_) => {
                        ctx.link().send_message(FileExplorerMsg::Refresh);
                        false
                    },
                    Err(e) => {
                        self.error_message = Some(e);
                        true
                    }
                }
            },
            FileExplorerMsg::CreateNewFile => {
                // This would typically open a dialog
                // For now, let's create a file with a default name
                let new_file_path = format!("{}/new_file.txt", self.current_path);
                match self.fs.borrow_mut().write_file(&new_file_path, "") {
                    Ok(_) => {
                        ctx.link().send_message(FileExplorerMsg::Refresh);
                        false
                    },
                    Err(e) => {
                        self.error_message = Some(e);
                        true
                    }
                }
            },
            FileExplorerMsg::CreateNewDirectory => {
                // This would typically open a dialog
                // For now, let's create a directory with a default name
                let new_dir_path = format!("{}/new_directory", self.current_path);
                match self.fs.borrow_mut().create_directory(&new_dir_path, false) {
                    Ok(_) => {
                        ctx.link().send_message(FileExplorerMsg::Refresh);
                        false
                    },
                    Err(e) => {
                        self.error_message = Some(e);
                        true
                    }
                }
            },
            FileExplorerMsg::Error(message) => {
                self.error_message = Some(message);
                true
            },
            FileExplorerMsg::ClearError => {
                self.error_message = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let path_parts: Vec<String> = self.current_path
            .split('/')
            .filter(|part| !part.is_empty())
            .map(|s| s.to_string())
            .collect();
        
        html! {
            <div class="file-explorer" style="display: flex; flex-direction: column; height: 100%;">
                // Path navigation
                <div class="path-bar" style="padding: 8px; background-color: #f0f0f0; border-bottom: 1px solid #ddd;">
                    <button onclick={ctx.link().callback(|_| FileExplorerMsg::NavigateUp)}>
                        { "↑ Up" }
                    </button>
                    <span style="margin-left: 8px;">
                        <button onclick={ctx.link().callback(|_| FileExplorerMsg::NavigateTo("/".to_string()))}>
                            { "/" }
                        </button>
                        {
                            path_parts.iter().enumerate().map(|(i, part)| {
                                let path = format!("/{}", path_parts[0..=i].join("/"));
                                html! {
                                    <>
                                        { " / " }
                                        <button onclick={ctx.link().callback(move |_| FileExplorerMsg::NavigateTo(path.clone()))}>
                                            { part }
                                        </button>
                                    </>
                                }
                            }).collect::<Html>()
                        }
                    </span>
                </div>
                
                // Toolbar
                <div class="toolbar" style="padding: 8px; background-color: #f8f8f8; border-bottom: 1px solid #ddd;">
                    <button onclick={ctx.link().callback(|_| FileExplorerMsg::Refresh)}>
                        { "Refresh" }
                    </button>
                    <button onclick={ctx.link().callback(|_| FileExplorerMsg::CreateNewFile)}>
                        { "New File" }
                    </button>
                    <button onclick={ctx.link().callback(|_| FileExplorerMsg::CreateNewDirectory)}>
                        { "New Directory" }
                    </button>
                </div>
                
                // Error messages
                {
                    if let Some(error) = &self.error_message {
                        html! {
                            <div class="error-message" style="padding: 8px; color: red; background-color: #fff0f0; border: 1px solid #ffdddd;">
                                { error }
                                <button onclick={ctx.link().callback(|_| FileExplorerMsg::ClearError)}>
                                    { "×" }
                                </button>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                
                // File list
                <div class="file-list" style="flex-grow: 1; overflow-y: auto; padding: 8px;">
                    <table style="width: 100%; border-collapse: collapse;">
                        <thead>
                            <tr style="background-color: #f5f5f5;">
                                <th style="text-align: left; padding: 8px; border-bottom: 1px solid #ddd;">{ "Name" }</th>
                                <th style="text-align: left; padding: 8px; border-bottom: 1px solid #ddd;">{ "Type" }</th>
                                <th style="text-align: right; padding: 8px; border-bottom: 1px solid #ddd;">{ "Size" }</th>
                                <th style="text-align: right; padding: 8px; border-bottom: 1px solid #ddd;">{ "Modified" }</th>
                                <th style="padding: 8px; border-bottom: 1px solid #ddd;">{ "Actions" }</th>
                            </tr>
                        </thead>
                        <tbody>
                            {
                                self.files.iter().map(|file| {
                                    let name = file.name.clone();
                                    let selected_style = if self.selected_file.as_ref() == Some(&name) {
                                        "background-color: #e0e8f0;"
                                    } else {
                                        ""
                                    };
                                    
                                    let type_icon = match file.file_type {
                                        FileType::Directory => "📁",
                                        FileType::File => "📄",
                                    };
                                    
                                    let type_name = match file.file_type {
                                        FileType::Directory => "Directory",
                                        FileType::File => "File",
                                    };
                                    
                                    let name_clone = name.clone();
                                    let name_clone2 = name.clone();
                                    
                                    let date = js_sys::Date::new(&JsValue::from_f64(file.modified as f64));
                                    let date_string = date.to_locale_string("en-US", &JsValue::undefined());
                                    
                                    html! {
                                        <tr style={selected_style} 
                                            onclick={ctx.link().callback(move |_| FileExplorerMsg::SelectFile(name_clone.clone()))}
                                            ondblclick={ctx.link().callback(move |_| FileExplorerMsg::OpenFile(name_clone2.clone()))}>
                                            <td style="padding: 8px; border-bottom: 1px solid #eee;">
                                                { type_icon } { " " } { &name }
                                            </td>
                                            <td style="padding: 8px; border-bottom: 1px solid #eee;">
                                                { type_name }
                                            </td>
                                            <td style="text-align: right; padding: 8px; border-bottom: 1px solid #eee;">
                                                {
                                                    match file.file_type {
                                                        FileType::Directory => html! { "" },
                                                        FileType::File => html! { format!("{} B", file.size) },
                                                    }
                                                }
                                            </td>
                                            <td style="text-align: right; padding: 8px; border-bottom: 1px solid #eee;">
                                                { date_string.as_string().unwrap_or_default() }
                                            </td>
                                            <td style="padding: 8px; border-bottom: 1px solid #eee;">
                                                <button onclick={ctx.link().callback(move |e: MouseEvent| {
                                                    e.stop_propagation();
                                                    FileExplorerMsg::DeleteFile(name.clone())
                                                })}>
                                                    { "Delete" }
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                }).collect::<Html>()
                            }
                        </tbody>
                    </table>
                </div>
            </div>
        }
    }
} 