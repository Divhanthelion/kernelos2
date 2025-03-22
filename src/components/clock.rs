use yew::prelude::*;
use wasm_bindgen::prelude::*;

pub struct Clock {
    time: String,
    date: String,
    _interval: Option<Interval>,
}

pub enum ClockMsg {
    Tick,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Interval {
    #[allow(dead_code)]
    id: i32,
}

impl Drop for Interval {
    fn drop(&mut self) {
        let window = web_sys::window().expect("no global `window` exists");
        window.clear_interval_with_handle(self.id);
    }
}

impl Component for Clock {
    type Message = ClockMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // Set up interval for automatic ticks
        let callback = ctx.link().callback(|_| ClockMsg::Tick);
        let handle = {
            let cb = Closure::wrap(Box::new(move || {
                callback.emit(());
            }) as Box<dyn FnMut()>);
            
            let window = web_sys::window().expect("should have a window in this context");
            let id = window
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    1000, // Update every second
                )
                .expect("failed to set interval");
            
            cb.forget(); // We need to forget it, otherwise it will be dropped
            id
        };

        let mut instance = Self {
            time: String::new(),
            date: String::new(),
            _interval: Some(Interval { id: handle }),
        };
        
        // Initialize the time
        instance.update_time();
        
        instance
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ClockMsg::Tick => {
                self.update_time();
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="clock" style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; background-color: #f7f7f7;">
                <div style="font-size: 5em; font-weight: 300; color: #333; text-align: center;">
                    { &self.time }
                </div>
                <div style="font-size: 1.5em; color: #777; margin-top: 20px; text-align: center;">
                    { &self.date }
                </div>
            </div>
        }
    }
}

impl Clock {
    fn update_time(&mut self) {
        let date = js_sys::Date::new_0();
        
        // Format time
        let hours = date.get_hours();
        let minutes = date.get_minutes();
        let seconds = date.get_seconds();
        let period = if hours >= 12 { "PM" } else { "AM" };
        let display_hours = if hours % 12 == 0 { 12 } else { hours % 12 };
        
        self.time = format!(
            "{:02}:{:02}:{:02} {}",
            display_hours, minutes, seconds, period
        );
        
        // Format date
        let days = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
        let months = [
            "January", "February", "March", "April", "May", "June",
            "July", "August", "September", "October", "November", "December"
        ];
        
        let day = days[date.get_day() as usize];
        let month = months[date.get_month() as usize];
        let date_num = date.get_date();
        let year = date.get_full_year();
        
        self.date = format!("{}, {} {}, {}", day, month, date_num, year);
    }
} 