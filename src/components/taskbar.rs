use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct TaskbarProps {
    pub windows: Vec<(String, String, bool)>, // (id, title, is_minimized)
    pub on_restore: Callback<String>,
    pub on_create_file_explorer: Callback<()>,
    pub on_create_terminal: Callback<()>,
    pub on_create_text_editor: Callback<()>,
    pub on_create_clock: Callback<()>,
}

pub struct Taskbar;

impl Component for Taskbar {
    type Message = ();
    type Properties = TaskbarProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let taskbar_style = "
            position: absolute;
            bottom: 0;
            left: 0;
            width: 100%;
            height: 48px;
            background-color: #333;
            display: flex;
            align-items: center;
            padding: 0 16px;
            box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.2);
        ";

        let start_button_style = "
            background-color: #4a86cf;
            border: none;
            color: white;
            padding: 8px 16px;
            border-radius: 4px;
            margin-right: 16px;
            cursor: pointer;
        ";

        let task_button_style = "
            background-color: #555;
            border: none;
            color: white;
            padding: 8px 16px;
            border-radius: 4px;
            margin-right: 8px;
            cursor: pointer;
            min-width: 100px;
            text-align: left;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        ";

        // Get current time for clock
        let now = js_sys::Date::new_0();
        let hours = now.get_hours();
        let minutes = now.get_minutes();
        let time_string = format!(
            "{:02}:{:02} {}",
            if hours % 12 == 0 { 12 } else { hours % 12 },
            minutes,
            if hours >= 12 { "PM" } else { "AM" }
        );

        html! {
            <div class="taskbar" style={taskbar_style}>
                <div class="start-menu">
                    <button style={start_button_style}>{ "Start" }</button>
                </div>
                
                <div class="quick-launch" style="margin-right: 16px;">
                    <button 
                        onclick={ctx.props().on_create_file_explorer.reform(|_| ())}
                        style="background: none; border: none; color: white; cursor: pointer; margin-right: 8px;"
                        title="File Explorer"
                    >
                        { "📁" }
                    </button>
                    <button 
                        onclick={ctx.props().on_create_terminal.reform(|_| ())}
                        style="background: none; border: none; color: white; cursor: pointer; margin-right: 8px;"
                        title="Terminal"
                    >
                        { "💻" }
                    </button>
                    <button 
                        onclick={ctx.props().on_create_text_editor.reform(|_| ())}
                        style="background: none; border: none; color: white; cursor: pointer; margin-right: 8px;"
                        title="Text Editor"
                    >
                        { "📝" }
                    </button>
                    <button 
                        onclick={ctx.props().on_create_clock.reform(|_| ())}
                        style="background: none; border: none; color: white; cursor: pointer;"
                        title="Clock"
                    >
                        { "🕒" }
                    </button>
                </div>
                
                <div class="window-buttons" style="display: flex; overflow-x: auto;">
                    {
                        ctx.props().windows.iter().map(|(id, title, is_minimized)| {
                            let id_clone = id.clone();
                            let button_style = if *is_minimized {
                                format!("{} opacity: 0.7;", task_button_style)
                            } else {
                                format!("{} background-color: #666;", task_button_style)
                            };
                            
                            html! {
                                <button 
                                    style={button_style}
                                    onclick={ctx.props().on_restore.reform(move |_| id_clone.clone())}
                                    title={title.clone()}
                                >
                                    { title }
                                </button>
                            }
                        }).collect::<Html>()
                    }
                </div>
                
                <div style="margin-left: auto; color: white;">
                    { time_string }
                </div>
            </div>
        }
    }
} 