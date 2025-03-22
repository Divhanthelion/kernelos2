use yew::prelude::*;
use web_sys::{self, MouseEvent};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
// Use WebWindow alias to avoid confusion with your component
use web_sys::Window as WebWindow;

use crate::components::window::{Window, WindowState, WindowContentType};
use crate::components::taskbar::Taskbar;
use crate::filesystem::FileSystem;

pub struct Desktop {
    fs: Rc<RefCell<FileSystem>>,
    windows: HashMap<String, Rc<RefCell<WindowState>>>,
    active_window_id: Option<String>,
    window_counter: u32,
    context_menu: Option<(i32, i32)>,
    background_color: String,
}

pub enum DesktopMsg {
    CreateWindow(String, WindowContentType),
    CloseWindow(String),
    MinimizeWindow(String),
    RestoreWindow(String),
    FocusWindow(String),
    ContextMenu(i32, i32),
    OpenFile(String, String), // (path, file_type)
    ChangeBackgroundColor(String),
}

impl Component for Desktop {
    type Message = DesktopMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        // Initialize file system
        let fs = match FileSystem::new() {
            Ok(fs) => Rc::new(RefCell::new(fs)),
            Err(e) => {
                log::error!("Failed to initialize file system: {}", e);
                // Rc::new(RefCell::new(FileSystem { // replaced with below - ensure this part is commented out
                //     files: HashMap::new(),
                // }))
                // In case of error, still try to create a FileSystem, even if it might be empty or in a bad state
                match FileSystem::new() {
                    Ok(fs_err) => Rc::new(RefCell::new(fs_err)),
                    Err(e_err) => {
                        log::error!("Failed to initialize file system AGAIN: {}", e_err);
                        // Rc::new(RefCell::new(FileSystem { files: HashMap::new() })) // REMOVE this line
                        // Fallback to creating a default FileSystem - if FileSystem implements Default
                        match FileSystem::new() { // Try one more time
                            Ok(fs_err_again) => Rc::new(RefCell::new(fs_err_again)),
                            Err(e_err_again) => {
                                log::error!("Failed to initialize file system AGAIN, AGAIN: {}", e_err_again);
                                // If even the second attempt fails, return an error or handle it appropriately.
                                // For now, let's panic to indicate a serious initialization failure.
                                panic!("Failed to initialize FileSystem after multiple attempts: {}", e_err_again);
                                // Alternatively, you could use a default FileSystem if you implement Default for FileSystem
                                // or return a Result<Desktop, String> from Desktop::create and handle the error in main.rs
                                // For a very basic fallback (if FileSystem implements Default):
                                // Rc::new(RefCell::new(FileSystem::default()))
                            }
                        }
                    }
                }
            }
        };

        Self {
            fs,
            windows: HashMap::new(),
            active_window_id: None,
            window_counter: 0,
            context_menu: None,
            background_color: "#2a6496".to_string(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            DesktopMsg::CreateWindow(title, content_type) => {
                let id = format!("window-{}", self.window_counter);
                self.window_counter += 1;
                
                // Position new windows in a cascading manner
                let x = 50 + ((self.windows.len() as i32 % 5) * 30);
                let y = 50 + ((self.windows.len() as i32 % 5) * 30);
                
                let window = Rc::new(RefCell::new(WindowState {
                    id: id.clone(),
                    title,
                    x,
                    y,
                    width: 600,
                    height: 400,
                    is_minimized: false,
                    is_focused: true,
                    content_type,
                }));
                
                // Unfocus all other windows
                for (_, other_window) in &self.windows {
                    other_window.borrow_mut().is_focused = false;
                }
                
                self.windows.insert(id.clone(), window);
                self.active_window_id = Some(id);
                true
            }
            DesktopMsg::CloseWindow(id) => {
                self.windows.remove(&id);
                
                // If we closed the active window, focus another one if available
                if self.active_window_id == Some(id.clone()) {
                    self.active_window_id = self.windows.keys().next().cloned();
                    if let Some(ref active_id) = self.active_window_id {
                        if let Some(window) = self.windows.get(active_id) {
                            window.borrow_mut().is_focused = true;
                        }
                    }
                }
                true
            }
            DesktopMsg::MinimizeWindow(id) => {
                if let Some(window) = self.windows.get(&id) {
                    window.borrow_mut().is_minimized = true;
                }
                true
            }
            DesktopMsg::RestoreWindow(id) => {
                if let Some(window) = self.windows.get(&id) {
                    let mut window = window.borrow_mut();
                    window.is_minimized = false;
                    window.is_focused = true;
                }
                
                // Unfocus all other windows
                for (window_id, window) in &self.windows {
                    if *window_id != id {
                        window.borrow_mut().is_focused = false;
                    }
                }
                
                self.active_window_id = Some(id);
                true
            }
            DesktopMsg::FocusWindow(id) => {
                // Unfocus all windows
                for (window_id, window) in &self.windows {
                    window.borrow_mut().is_focused = *window_id == id;
                }
                
                self.active_window_id = Some(id);
                true
            }
            DesktopMsg::ContextMenu(x, y) => {
                // Toggle context menu
                if self.context_menu.is_some() {
                    self.context_menu = None;
                } else {
                    self.context_menu = Some((x, y));
                }
                true
            }
            DesktopMsg::OpenFile(path, file_type) => {
                // Open file in appropriate application
                let content_type = match file_type.as_str() {
                    "text" => WindowContentType::TextEditor { file_path: Some(path.clone()) },
                    "image" => WindowContentType::ImageViewer { file_path: path.clone() },
                    _ => WindowContentType::TextEditor { file_path: Some(path.clone()) }, // Default to text editor
                };
                
                // Create window title from file path
                let title = match Path::new(&path).file_name() {
                    Some(name) => format!("{} - {}", name.to_string_lossy(), 
                        if file_type == "text" { "Text Editor" } else { "Image Viewer" }),
                    None => format!("{} - {}", path, 
                        if file_type == "text" { "Text Editor" } else { "Image Viewer" }),
                };
                
                _ctx.link().send_message(DesktopMsg::CreateWindow(title, content_type));
                false
            }
            DesktopMsg::ChangeBackgroundColor(color) => {
                self.background_color = color;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_close = ctx.link().callback(DesktopMsg::CloseWindow);
        let on_minimize = ctx.link().callback(DesktopMsg::MinimizeWindow);
        let on_focus = ctx.link().callback(DesktopMsg::FocusWindow);
        let on_restore = ctx.link().callback(DesktopMsg::RestoreWindow);
        
        let on_context_menu = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            DesktopMsg::ContextMenu(e.client_x(), e.client_y())
        });
        
        // Define main callbacks first
        let create_file_explorer = ctx.link().callback(|_| {
            DesktopMsg::CreateWindow("File Explorer".to_string(), WindowContentType::FileExplorer)
        });

        let create_terminal = ctx.link().callback(|_| {
            DesktopMsg::CreateWindow("Terminal".to_string(), WindowContentType::Terminal)
        });

        let create_text_editor = ctx.link().callback(|_| {
            DesktopMsg::CreateWindow(
            "Text Editor".to_string(), 
                WindowContentType::TextEditor { file_path: None }
            )
        });

        let create_clock = ctx.link().callback(|_| {
            DesktopMsg::CreateWindow("Clock".to_string(), WindowContentType::Clock)
        });

// Create context menu callbacks that accept MouseEvent parameters
        let create_file_explorer_clone = create_file_explorer.clone();
        let create_file_explorer_ctx = Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            create_file_explorer_clone.emit(())
       });

        let create_terminal_clone = create_terminal.clone();
        let create_terminal_ctx = Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            create_terminal_clone.emit(())
        });

        let create_text_editor_clone = create_text_editor.clone();
        let create_text_editor_ctx = Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            create_text_editor_clone.emit(())
        });

        let create_clock_clone = create_clock.clone();
        let create_clock_ctx = Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            create_clock_clone.emit(())
        });

        let fs_clone = Rc::clone(&self.fs);
        let on_open_file = ctx.link().callback(move |(path, file_type)| {
            DesktopMsg::OpenFile(path, file_type)
        });
        
        // Context menu click handlers
        let hide_context_menu = ctx.link().callback(|_| DesktopMsg::ContextMenu(0, 0));
        let create_file_explorer_ctx = create_file_explorer.clone();
        let create_terminal_ctx = create_terminal.clone();
        let create_text_editor_ctx = create_text_editor.clone();
        let create_clock_ctx = create_clock.clone();
        let create_file_compressor = ctx.link().callback(|_| {
            DesktopMsg::CreateWindow("File Compressor".to_string(), WindowContentType::FileCompressor)
        });
        
        // Background color callbacks
        let blue_bg = ctx.link().callback(|_| DesktopMsg::ChangeBackgroundColor("#2a6496".to_string()));
        let green_bg = ctx.link().callback(|_| DesktopMsg::ChangeBackgroundColor("#2a9652".to_string()));
        let purple_bg = ctx.link().callback(|_| DesktopMsg::ChangeBackgroundColor("#5c2a96".to_string()));
        let dark_bg = ctx.link().callback(|_| DesktopMsg::ChangeBackgroundColor("#1a1a2e".to_string()));

        html! {
            <>
                <div class="desktop" 
                     style={format!("width: 100%; height: 100vh; background-color: {}; position: relative; overflow: hidden;", self.background_color)}
                     oncontextmenu={on_context_menu}>
                    
                    /* Windows */
                    {
                        self.windows.iter().map(|(_, window)| {
                            html! {
                                <Window 
                                    window={Rc::clone(window)}
                                    fs={Rc::clone(&self.fs)}
                                    on_close={on_close.clone()}
                                    on_minimize={on_minimize.clone()}
                                    on_focus={on_focus.clone()}
                                    on_open_file={on_open_file.clone()}
                                />
                            }
                        }).collect::<Html>()
                    }
                    
                    // Taskbar
                    <Taskbar 
                        windows={
                            self.windows.iter().map(|(id, w)| {
                                (id.clone(), w.borrow().title.clone(), w.borrow().is_minimized)
                            }).collect::<Vec<_>>()
                        }
                        on_restore={on_restore}
                        on_create_file_explorer={create_file_explorer}
                        on_create_terminal={create_terminal}
                        on_create_text_editor={create_text_editor}
                        on_create_clock={create_clock}
                    />
                    
                    // Context Menu (conditionally rendered)
                    {
                        if let Some((x, y)) = self.context_menu {
                            let menu_style = format!(
                                "position: absolute; left: {}px; top: {}px; background-color: white; 
                                 border: 1px solid #ccc; border-radius: 4px; box-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
                                 z-index: 100;",
                                x, y
                            );
                            
                            let menu_item_style = 
                                "padding: 8px 16px; cursor: pointer; white-space: nowrap; 
                                 user-select: none; display: flex; align-items: center;";
                            
                            let hover_style = "hover:background-color: #f0f0f0;";
                            
                            html! {
                                <>
                                    <div class="context-menu-overlay" 
                                         style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 99;"
                                         onclick={hide_context_menu.clone()}>
                                    </div>
                                    <div class="context-menu" style={menu_style}>
                                        <div class="context-menu-item" 
                                             style={menu_item_style}>
                                            <span style="margin-right: 8px;">{"📁"}</span>
                                            {"Open File Explorer"}
                                        </div>
                                        <div class="context-menu-item"
                                             style={menu_item_style}>
                                            <span style="margin-right: 8px;">{"💻"}</span>
                                            {"Open Terminal"}
                                        </div>
                                        <div class="context-menu-item"
                                             style={menu_item_style}>
                                            <span style="margin-right: 8px;">{"📝"}</span>
                                            {"New Text Document"}
                                        </div>
                                        <div class="context-menu-item"
                                             style={menu_item_style}>
                                            <span style="margin-right: 8px;">{"🕒"}</span>
                                            {"Open Clock"}
                                        </div>
                                        <div class="context-menu-item"
                                             style={menu_item_style}
                                             onclick={create_file_compressor}>
                                            <span style="margin-right: 8px;">{"🗜️"}</span>
                                            {"File Compressor"}
                                        </div>
                                        <hr style="margin: 4px 0; border-top: 1px solid #eee;" />
                                        <div class="context-menu-item"
                                             style={menu_item_style}>
                                            <span style="margin-right: 8px;">{"🎨"}</span>
                                            {"Change Background"}
                                            <div style="display: flex; margin-left: 8px;">
                                                <div 
                                                    style="width: 16px; height: 16px; background-color: #2a6496; margin-right: 4px; cursor: pointer; border: 1px solid #ccc;" 
                                                    onclick={blue_bg}
                                                ></div>
                                                <div 
                                                    style="width: 16px; height: 16px; background-color: #2a9652; margin-right: 4px; cursor: pointer; border: 1px solid #ccc;" 
                                                    onclick={green_bg}
                                                ></div>
                                                <div 
                                                    style="width: 16px; height: 16px; background-color: #5c2a96; margin-right: 4px; cursor: pointer; border: 1px solid #ccc;" 
                                                    onclick={purple_bg}
                                                ></div>
                                                <div 
                                                    style="width: 16px; height: 16px; background-color: #1a1a2e; cursor: pointer; border: 1px solid #ccc;" 
                                                    onclick={dark_bg}
                                                ></div>
                                            </div>
                                        </div>
                                        <hr style="margin: 4px 0; border-top: 1px solid #eee;" />
                                        <div class="context-menu-item"
                                             style={menu_item_style}
                                             onclick={hide_context_menu}>
                                            {"Cancel"}
                                        </div>
                                    </div>
                                </>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            </>
        }   
    }
}