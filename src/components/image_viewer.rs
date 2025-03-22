use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::filesystem::FileSystem;

pub struct ImageViewer {
    fs: Rc<RefCell<FileSystem>>,
    file_path: String,
    error_message: Option<String>,
    zoom_level: f64,
    image_data: Option<String>,
}

pub enum ImageViewerMsg {
    ZoomIn,
    ZoomOut,
    ResetZoom,
    SetError(String),
    ClearError,
}

#[derive(Properties, Clone, PartialEq)]
pub struct ImageViewerProps {
    pub fs: Rc<RefCell<FileSystem>>,
    pub file_path: String,
}

impl Component for ImageViewer {
    type Message = ImageViewerMsg;
    type Properties = ImageViewerProps;

    fn create(ctx: &Context<Self>) -> Self {
        let fs = Rc::clone(&ctx.props().fs);
        let file_path = ctx.props().file_path.clone();
        
        // In a real implementation, we would load the actual image data
        // For this simplified version, we'll just simulate an image viewer
        // by showing a placeholder and the file path
        
        Self {
            fs,
            file_path,
            error_message: None,
            zoom_level: 1.0,
            image_data: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ImageViewerMsg::ZoomIn => {
                self.zoom_level = (self.zoom_level * 1.2).min(5.0);
                true
            }
            ImageViewerMsg::ZoomOut => {
                self.zoom_level = (self.zoom_level / 1.2).max(0.1);
                true
            }
            ImageViewerMsg::ResetZoom => {
                self.zoom_level = 1.0;
                true
            }
            ImageViewerMsg::SetError(message) => {
                self.error_message = Some(message);
                true
            }
            ImageViewerMsg::ClearError => {
                self.error_message = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let file_name = std::path::Path::new(&self.file_path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        html! {
            <div class="image-viewer" style="display: flex; flex-direction: column; height: 100%;">
                <div class="toolbar" style="padding: 8px; background-color: #f0f0f0; border-bottom: 1px solid #ddd; display: flex; justify-content: space-between;">
                    <div>
                        <button onclick={ctx.link().callback(|_| ImageViewerMsg::ZoomIn)}>
                            { "Zoom In" }
                        </button>
                        <button onclick={ctx.link().callback(|_| ImageViewerMsg::ZoomOut)} style="margin-left: 8px;">
                            { "Zoom Out" }
                        </button>
                        <button onclick={ctx.link().callback(|_| ImageViewerMsg::ResetZoom)} style="margin-left: 8px;">
                            { "Reset Zoom" }
                        </button>
                    </div>
                    <div>
                        <span>{ format!("Zoom: {}%", (self.zoom_level * 100.0) as i32) }</span>
                    </div>
                </div>
                
                {
                    if let Some(error) = &self.error_message {
                        html! {
                            <div class="error-message" style="padding: 8px; color: red; background-color: #fff0f0; border-bottom: 1px solid #ffdddd;">
                                { error }
                                <button 
                                    style="margin-left: 8px;" 
                                    onclick={ctx.link().callback(|_| ImageViewerMsg::ClearError)}
                                >
                                    { "×" }
                                </button>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                
                <div class="image-container" style="flex-grow: 1; overflow: auto; display: flex; align-items: center; justify-content: center; background-color: #222;">
                    <div style="text-align: center;">
                        // For a real implementation, we would load and display the actual image
                        // Here we're just showing a placeholder
                        <div style={format!("width: 300px; height: 200px; background-color: #444; display: flex; align-items: center; justify-content: center; color: white; transform: scale({}); transition: transform 0.2s ease-in-out;", self.zoom_level)}>
                            { "Image Placeholder" }
                        </div>
                        <div style="margin-top: 16px; color: white;">
                            { format!("File: {}", file_name) }
                        </div>
                        <div style="margin-top: 8px; color: #aaa; font-size: 0.9em;">
                            { "Note: In a real implementation, images would be loaded and displayed here." }
                        </div>
                    </div>
                </div>
            </div>
        }
    }
} 