use yew::prelude::*;
use web_sys::{HtmlTextAreaElement, KeyboardEvent};
use std::rc::Rc;
use std::cell::RefCell;
use crate::filesystem::FileSystem;

pub struct TextEditor {
    fs: Rc<RefCell<FileSystem>>,
    file_path: Option<String>,
    content: String,
    is_modified: bool,
    error_message: Option<String>,
    textarea_ref: NodeRef,
}

pub enum TextEditorMsg {
    ContentChanged(String),
    SaveFile,
    KeyDown(KeyboardEvent),
    SetError(String),
    ClearError,
}

#[derive(Properties, Clone, PartialEq)]
pub struct TextEditorProps {
    pub fs: Rc<RefCell<FileSystem>>,
    pub file_path: Option<String>,
}

impl Component for TextEditor {
    type Message = TextEditorMsg;
    type Properties = TextEditorProps;

    fn create(ctx: &Context<Self>) -> Self {
        let fs = Rc::clone(&ctx.props().fs);
        let file_path = ctx.props().file_path.clone();
        
        // Load file content if file path is provided
        let content = if let Some(path) = &file_path {
            match fs.borrow().read_file(path) {
                Ok(content) => content,
                Err(e) => {
                    log::error!("Failed to load file {}: {}", path, e);
                    String::new()
                }
            }
        } else {
            String::new()
        };

        Self {
            fs,
            file_path,
            content,
            is_modified: false,
            error_message: None,
            textarea_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TextEditorMsg::ContentChanged(new_content) => {
                let changed = new_content != self.content;
                self.content = new_content;
                if changed {
                    self.is_modified = true;
                }
                true
            }
            TextEditorMsg::SaveFile => {
                if let Some(path) = &self.file_path {
                    match self.fs.borrow_mut().write_file(path, &self.content) {
                        Ok(_) => {
                            self.is_modified = false;
                            true
                        }
                        Err(e) => {
                            ctx.link().send_message(TextEditorMsg::SetError(format!("Failed to save file: {}", e)));
                            false
                        }
                    }
                } else {
                    // Would typically open a save dialog
                    // For now, let's save to a default path
                    let default_path = "/home/documents/untitled.txt";
                    match self.fs.borrow_mut().write_file(default_path, &self.content) {
                        Ok(_) => {
                            self.file_path = Some(default_path.to_string());
                            self.is_modified = false;
                            true
                        }
                        Err(e) => {
                            ctx.link().send_message(TextEditorMsg::SetError(format!("Failed to save file: {}", e)));
                            false
                        }
                    }
                }
            }
            TextEditorMsg::KeyDown(event) => {
                // Check for Ctrl+S
                if event.ctrl_key() && event.key() == "s" {
                    event.prevent_default();
                    ctx.link().send_message(TextEditorMsg::SaveFile);
                    true
                } else {
                    false
                }
            }
            TextEditorMsg::SetError(message) => {
                self.error_message = Some(message);
                true
            }
            TextEditorMsg::ClearError => {
                self.error_message = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(|e: InputEvent| {
            let textarea: HtmlTextAreaElement = e.target_unchecked_into();
            TextEditorMsg::ContentChanged(textarea.value())
        });
        
        let onkeydown = ctx.link().callback(TextEditorMsg::KeyDown);
        
        let title = match &self.file_path {
            Some(path) => {
                let file_name = std::path::Path::new(path)
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Untitled".to_string());
                
                if self.is_modified {
                    format!("*{}", file_name)
                } else {
                    file_name
                }
            }
            None => {
                if self.is_modified {
                    "*Untitled".to_string()
                } else {
                    "Untitled".to_string()
                }
            }
        };
        
        html! {
            <div class="text-editor" style="display: flex; flex-direction: column; height: 100%;">
                <div class="toolbar" style="padding: 8px; background-color: #f0f0f0; border-bottom: 1px solid #ddd; display: flex; justify-content: space-between;">
                    <div>
                        <button onclick={ctx.link().callback(|_| TextEditorMsg::SaveFile)}>
                            { "Save" }
                        </button>
                        <span style="margin-left: 16px;">{ title }</span>
                    </div>
                    <div>
                        <span style="color: #777; font-size: 0.9em;">{ "Ctrl+S to save" }</span>
                    </div>
                </div>
                
                {
                    if let Some(error) = &self.error_message {
                        html! {
                            <div class="error-message" style="padding: 8px; color: red; background-color: #fff0f0; border-bottom: 1px solid #ffdddd;">
                                { error }
                                <button 
                                    style="margin-left: 8px;" 
                                    onclick={ctx.link().callback(|_| TextEditorMsg::ClearError)}
                                >
                                    { "×" }
                                </button>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                
                <textarea
                    style="flex-grow: 1; resize: none; padding: 8px; font-family: monospace; border: none; outline: none; background-color: white; color: #333;"
                    value={self.content.clone()}
                    ref={self.textarea_ref.clone()}
                    {oninput}
                    {onkeydown}
                    spellcheck="false"
                />
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            // Focus textarea on first render
            if let Some(textarea) = self.textarea_ref.cast::<HtmlTextAreaElement>() {
                let _ = textarea.focus();
            }
        }
    }
} 