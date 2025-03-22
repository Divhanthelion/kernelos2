use yew::prelude::*;
use web_sys::{MouseEvent};
use std::rc::Rc;
use std::cell::RefCell;

use crate::filesystem::FileSystem;
use crate::components::terminal::Terminal;
use crate::components::file_explorer::FileExplorer;
use crate::components::text_editor::TextEditor;
use crate::components::clock::Clock;
use crate::components::image_viewer::ImageViewer;
use crate::components::file_compressor::FileCompressor;

// Window state
#[derive(Debug, Clone, PartialEq)]
pub struct WindowState {
    pub id: String,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub is_minimized: bool,
    pub is_focused: bool,
    pub content_type: WindowContentType,
}

// Different types of window content
#[derive(Clone, PartialEq, Debug)]
pub enum WindowContentType {
    Empty,
    Terminal,
    FileExplorer,
    TextEditor { file_path: Option<String> },
    Clock,
    ImageViewer { file_path: String },
    FileCompressor,
}

// Properties for the Window component
#[derive(Properties, Clone, PartialEq)]
pub struct WindowProps {
    pub window: Rc<RefCell<WindowState>>,
    pub fs: Rc<RefCell<FileSystem>>,
    pub on_close: Callback<String>,
    pub on_focus: Callback<String>,
    pub on_minimize: Callback<String>,
    pub on_open_file: Callback<(String, String)>,
}

// Window component
pub struct Window {
    is_dragging: bool,
    drag_start_x: i32,
    drag_start_y: i32,
    window_start_x: i32,
    window_start_y: i32,
    node_ref: NodeRef,
}

pub enum WindowMsg {
    StartDrag(i32, i32),
    Drag(i32, i32),
    StopDrag,
    Close,
    Minimize,
    Focus,
}

impl Component for Window {
    type Message = WindowMsg;
    type Properties = WindowProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            is_dragging: false,
            drag_start_x: 0,
            drag_start_y: 0,
            window_start_x: 0,
            window_start_y: 0,
            node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            WindowMsg::StartDrag(x, y) => {
                let window = ctx.props().window.borrow();
                self.is_dragging = true;
                self.drag_start_x = x;
                self.drag_start_y = y;
                self.window_start_x = window.x;
                self.window_start_y = window.y;
                true
            }
            WindowMsg::Drag(x, y) => {
                if self.is_dragging {
                    let mut window = ctx.props().window.borrow_mut();
                    window.x = self.window_start_x + (x - self.drag_start_x);
                    window.y = self.window_start_y + (y - self.drag_start_y);
                    true
                } else {
                    false
                }
            }
            WindowMsg::StopDrag => {
                self.is_dragging = false;
                true
            }
            WindowMsg::Close => {
                ctx.props().on_close.emit(ctx.props().window.borrow().id.clone());
                false
            }
            WindowMsg::Minimize => {
                ctx.props().on_minimize.emit(ctx.props().window.borrow().id.clone());
                false
            }
            WindowMsg::Focus => {
                ctx.props().on_focus.emit(ctx.props().window.borrow().id.clone());
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let window = ctx.props().window.borrow();
        
        let window_style = format!(
            "position: absolute; left: {}px; top: {}px; width: {}px; height: {}px; 
             z-index: {}; display: {}; border-radius: 8px; overflow: hidden; box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);",
            window.x, window.y, window.width, window.height,
            if window.is_focused { "10" } else { "5" },
            if window.is_minimized { "none" } else { "block" }
        );

        let title_bar_style = "padding: 10px; background-color: #4a4a4a; color: white; cursor: move; display: flex; justify-content: space-between; align-items: center;";
        
        let onmousedown = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            WindowMsg::StartDrag(e.client_x(), e.client_y())
        });
        
        let onmousemove = ctx.link().callback(|e: MouseEvent| {
            WindowMsg::Drag(e.client_x(), e.client_y())
        });
        
        let onmouseup = ctx.link().callback(|_| WindowMsg::StopDrag);
        let onmouseleave = ctx.link().callback(|_| WindowMsg::StopDrag);
        
        let onclick = ctx.link().callback(|_| WindowMsg::Focus);
        let on_close = ctx.link().callback(|_| WindowMsg::Close);
        let on_minimize = ctx.link().callback(|_| WindowMsg::Minimize);

        html! {
            <div class="window" style={window_style} onclick={onclick} ref={self.node_ref.clone()}>
                <div class="window-titlebar" 
                     style={title_bar_style}
                     onmousedown={onmousedown}
                     onmousemove={onmousemove}
                     onmouseup={onmouseup}
                     onmouseleave={onmouseleave}>
                    <span style="font-weight: bold;">{ &window.title }</span>
                    <div>
                        <button style="background: none; border: none; color: white; margin-right: 8px; cursor: pointer;" 
                                onclick={on_minimize}>{"_"}</button>
                        <button style="background: none; border: none; color: white; cursor: pointer;"
                                onclick={on_close}>{"×"}</button>
                    </div>
                </div>
                <div class="window-content" style="background-color: #f5f5f5; height: calc(100% - 40px); overflow: auto;">
                    { self.render_content(ctx) }
                </div>
            </div>
        }
    }
}

impl Window {
    fn render_content(&self, ctx: &Context<Self>) -> Html {
        let window = ctx.props().window.borrow();
        let fs = Rc::clone(&ctx.props().fs);
        let on_open_file = ctx.props().on_open_file.clone();
        
        match &window.content_type {
            WindowContentType::Empty => html! {},
            WindowContentType::Terminal => {
                html! { <Terminal fs={fs} /> }
            }
            WindowContentType::FileExplorer => {
                html! { <FileExplorer fs={fs} on_open_file={on_open_file} /> }
            }
            WindowContentType::TextEditor { file_path } => {
                html! { <TextEditor fs={fs} file_path={file_path.clone()} /> }
            }
            WindowContentType::Clock => {
                html! { <Clock /> }
            }
            WindowContentType::ImageViewer { file_path } => {
                html! { <ImageViewer fs={fs} file_path={file_path.clone()} /> }
            }
            WindowContentType::FileCompressor => {
                html! { <FileCompressor fs={fs} /> }
            }
        }
    }
} 