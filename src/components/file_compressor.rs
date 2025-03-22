use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::filesystem::{FileSystem, FileType};
use std::path::Path;

pub struct FileCompressor {
    fs: Rc<RefCell<FileSystem>>,
    current_directory: String,
    selected_files: Vec<String>,
    archive_name: String,
    status_message: Option<(String, bool)>, // (message, is_error)
}

pub enum FileCompressorMsg {
    NavigateTo(String),
    NavigateUp,
    Refresh,
    ToggleFileSelection(String),
    UpdateArchiveName(String),
    CompressFiles,
    ExtractArchive(String),
    ClearMessage,
}

#[derive(Properties, Clone, PartialEq)]
pub struct FileCompressorProps {
    pub fs: Rc<RefCell<FileSystem>>,
}

impl Component for FileCompressor {
    type Message = FileCompressorMsg;
    type Properties = FileCompressorProps;

    fn create(ctx: &Context<Self>) -> Self {
        let fs = Rc::clone(&ctx.props().fs);
        
        Self {
            fs,
            current_directory: "/home".to_string(),
            selected_files: Vec::new(),
            archive_name: "archive.zip".to_string(),
            status_message: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FileCompressorMsg::NavigateTo(path) => {
                match self.fs.borrow().list_directory(&path) {
                    Ok(_) => {
                        self.current_directory = path;
                        self.selected_files.clear();
                        true
                    },
                    Err(e) => {
                        self.status_message = Some((format!("Error: {}", e), true));
                        true
                    }
                }
            },
            FileCompressorMsg::NavigateUp => {
                let parent = Path::new(&self.current_directory)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "/".to_string());
                
                ctx.link().send_message(FileCompressorMsg::NavigateTo(parent));
                false
            },
            FileCompressorMsg::Refresh => {
                self.status_message = None;
                true
            },
            FileCompressorMsg::ToggleFileSelection(file_path) => {
                if self.selected_files.contains(&file_path) {
                    self.selected_files.retain(|p| p != &file_path);
                } else {
                    self.selected_files.push(file_path);
                }
                true
            },
            FileCompressorMsg::UpdateArchiveName(name) => {
                self.archive_name = name;
                true
            },
            FileCompressorMsg::CompressFiles => {
                if self.selected_files.is_empty() {
                    self.status_message = Some(("No files selected for compression".to_string(), true));
                    return true;
                }
                
                // Basic implementation: In a real implementation, we would use a compression library
                // Here, we'll simulate compression by creating a new file with a list of files
                let archive_path = if self.archive_name.ends_with(".zip") {
                    format!("{}/{}", self.current_directory, self.archive_name)
                } else {
                    format!("{}/{}.zip", self.current_directory, self.archive_name)
                };
                
                // Simple text representation of the archive
                let archive_content = format!(
                    "SIMULATED ZIP ARCHIVE\n\
                     Created: {}\n\
                     Files:\n{}", 
                    js_sys::Date::new_0().to_string(),
                    self.selected_files.iter().map(|f| format!(" - {}\n", f)).collect::<String>()
                );
                
                match self.fs.borrow_mut().write_file(&archive_path, &archive_content) {
                    Ok(_) => {
                        self.status_message = Some((format!("Successfully created archive: {}", archive_path), false));
                        self.selected_files.clear();
                    },
                    Err(e) => {
                        self.status_message = Some((format!("Failed to create archive: {}", e), true));
                    }
                }
                
                true
            },
            FileCompressorMsg::ExtractArchive(path) => {
                // Simple extraction simulation
                match self.fs.borrow().read_file(&path) {
                    Ok(content) => {
                        if content.starts_with("SIMULATED ZIP ARCHIVE") {
                            // Extract archive name without extension
                            let archive_name = Path::new(&path)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("extracted");
                            
                            // Create extract directory
                            let extract_dir = format!("{}/{}_extracted", self.current_directory, archive_name);
                            
                            match self.fs.borrow_mut().create_directory(&extract_dir, true) {
                                Ok(_) => {
                                    // Create a sample extracted file
                                    let sample_file = format!("{}/README.txt", extract_dir);
                                    match self.fs.borrow_mut().write_file(&sample_file, "This is a simulated extracted file.\nIn a real implementation, the actual files would be extracted here.") {
                                        Ok(_) => {
                                            self.status_message = Some((format!("Extracted to: {}", extract_dir), false));
                                        },
                                        Err(e) => {
                                            self.status_message = Some((format!("Failed to create extracted file: {}", e), true));
                                        }
                                    }
                                },
                                Err(e) => {
                                    self.status_message = Some((format!("Failed to create extraction directory: {}", e), true));
                                }
                            }
                        } else {
                            self.status_message = Some(("Not a valid zip archive".to_string(), true));
                        }
                    },
                    Err(e) => {
                        self.status_message = Some((format!("Failed to read archive: {}", e), true));
                    }
                }
                
                true
            },
            FileCompressorMsg::ClearMessage => {
                self.status_message = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let files = match self.fs.borrow().list_directory(&self.current_directory) {
            Ok(files) => files,
            Err(_) => Vec::new(),
        };
        
        let path_parts: Vec<String> = self.current_directory
            .split('/')
            .filter(|part| !part.is_empty())
            .map(|s| s.to_string())
            .collect();
        
        html! {
            <div class="file-compressor" style="display: flex; flex-direction: column; height: 100%;">
                <div class="path-bar" style="padding: 8px; background-color: #f0f0f0; border-bottom: 1px solid #ddd;">
                    <button onclick={ctx.link().callback(|_| FileCompressorMsg::NavigateUp)}>
                        { "↑ Up" }
                    </button>
                    <span style="margin-left: 8px;">
                        <button onclick={ctx.link().callback(|_| FileCompressorMsg::NavigateTo("/".to_string()))}>
                            { "/" }
                        </button>
                        {
                            path_parts.iter().enumerate().map(|(i, part)| {
                                let path = format!("/{}", path_parts[0..=i].join("/"));
                                html! {
                                    <>
                                        { " / " }
                                        <button onclick={ctx.link().callback(move |_| FileCompressorMsg::NavigateTo(path.clone()))}>
                                            { part }
                                        </button>
                                    </>
                                }
                            }).collect::<Html>()
                        }
                    </span>
                </div>
                
                {
                    if let Some((message, is_error)) = &self.status_message {
                        let style = if *is_error {
                            "padding: 8px; background-color: #ffebee; color: #d32f2f; margin-bottom: 8px;"
                        } else {
                            "padding: 8px; background-color: #e8f5e9; color: #388e3c; margin-bottom: 8px;"
                        };
                        
                        html! {
                            <div style={style}>
                                { message }
                                <button 
                                    style="margin-left: 8px; background: none; border: none; cursor: pointer;"
                                    onclick={ctx.link().callback(|_| FileCompressorMsg::ClearMessage)}
                                >
                                    { "×" }
                                </button>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                
                <div class="compression-controls" style="padding: 8px; background-color: #f9f9f9; border-bottom: 1px solid #ddd;">
                    <div style="margin-bottom: 8px;">
                        <label style="margin-right: 8px;">{ "Archive Name:" }</label>
                        <input 
                            type="text" 
                            value={self.archive_name.clone()} 
                            onchange={ctx.link().callback(|e: Event| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                FileCompressorMsg::UpdateArchiveName(input.value())
                            })}
                        />
                    </div>
                    
                    <button 
                        disabled={self.selected_files.is_empty()}
                        onclick={ctx.link().callback(|_| FileCompressorMsg::CompressFiles)}
                        style={if self.selected_files.is_empty() { 
                            "opacity: 0.6; cursor: not-allowed;" 
                        } else { 
                            "cursor: pointer;" 
                        }}
                    >
                        { "Compress Selected Files" }
                    </button>
                    
                    <div style="margin-top: 8px;">
                        { format!("Selected: {} files", self.selected_files.len()) }
                    </div>
                </div>
                
                <div class="file-list" style="flex-grow: 1; overflow-y: auto; padding: 8px;">
                    <table style="width: 100%; border-collapse: collapse;">
                        <thead>
                            <tr style="background-color: #f0f0f0; text-align: left;">
                                <th style="padding: 8px; border-bottom: 1px solid #ddd;">{ "Select" }</th>
                                <th style="padding: 8px; border-bottom: 1px solid #ddd;">{ "Name" }</th>
                                <th style="padding: 8px; border-bottom: 1px solid #ddd;">{ "Type" }</th>
                                <th style="padding: 8px; border-bottom: 1px solid #ddd;">{ "Actions" }</th>
                            </tr>
                        </thead>
                        <tbody>
                            {
                                files.iter().map(|file| {
                                    let file_path = format!("{}/{}", self.current_directory, file.name);
                                    let file_path_clone = file_path.clone();
                                    let file_type = match file.file_type {
                                        FileType::Directory => "Directory",
                                        FileType::File => {
                                            if file.name.ends_with(".zip") {
                                                "Archive"
                                            } else {
                                                "File"
                                            }
                                        },
                                    };
                                    
                                    let is_selected = self.selected_files.contains(&file_path);
                                    let is_dir = matches!(file.file_type, FileType::Directory);
                                    let is_archive = file.name.ends_with(".zip");
                                    
                                    let file_path_for_nav = file_path.clone();
                                    let file_name = file.name.clone();
                                    
                                    html! {
                                        <tr style="border-bottom: 1px solid #f0f0f0;">
                                            <td style="padding: 8px;">
                                                <input 
                                                    type="checkbox" 
                                                    checked={is_selected}
                                                    disabled={is_dir}
                                                    onchange={ctx.link().callback(move |_| {
                                                        FileCompressorMsg::ToggleFileSelection(file_path_clone.clone())
                                                    })}
                                                />
                                            </td>
                                            <td style="padding: 8px;">
                                                <div onclick={
                                                    if is_dir {
                                                        ctx.link().callback(move |_| {
                                                            FileCompressorMsg::NavigateTo(file_path_for_nav.clone())
                                                        })
                                                    } else {
                                                        ctx.link().callback(|_| FileCompressorMsg::Refresh)
                                                    }
                                                } style={if is_dir { "cursor: pointer;" } else { "" }}>
                                                    {
                                                        if is_dir {
                                                            html! { <span>{ "📁 " }{ file_name.clone() }</span> }
                                                        } else if is_archive {
                                                            html! { <span>{ "🗜️ " }{ file_name.clone() }</span> }
                                                        } else {
                                                            html! { <span>{ "📄 " }{ file_name.clone() }</span> }
                                                        }
                                                    }
                                                </div>
                                            </td>
                                            <td style="padding: 8px;">{ file_type }</td>
                                            <td style="padding: 8px;">
                                                {
                                                    if is_archive {
                                                        let extract_path = file_path.clone();
                                                        html! {
                                                            <button onclick={ctx.link().callback(move |_| {
                                                                FileCompressorMsg::ExtractArchive(extract_path.clone())
                                                            })}>
                                                                { "Extract" }
                                                            </button>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
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
