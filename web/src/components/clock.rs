use gloo::timers::callback::Timeout;
use time::OffsetDateTime;
use yew::prelude::*;

pub struct Clock {
    time: OffsetDateTime,
    timeout: Timeout,
}

pub enum ClockMsg {
    Tick,
}

impl Component for Clock {
    type Message = ClockMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let time = OffsetDateTime::now_local().unwrap();

        let timeout = {
            let link = ctx.link().clone();
            let ms: u32 = time.millisecond().into();
            Timeout::new(1000 - ms, move || link.send_message(ClockMsg::Tick))
        };

        Self { time, timeout }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        // Prevent accidental changes to the Message type.
        let ClockMsg::Tick = msg;

        let time = OffsetDateTime::now_local().unwrap();

        let timeout = {
            let link = ctx.link().clone();
            let ms: u32 = time.millisecond().into();
            Timeout::new(1000 - ms, move || link.send_message(ClockMsg::Tick))
        };

        self.time = time;
        self.timeout = timeout;
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        use time::macros::format_description;

        let date = format_description!("[weekday], [day padding:none] [month repr:long] [year]");
        let time = format_description!("[hour repr:24]:[minute]:[second]");
        let iso = format_description!(
            "[year]-[month]-[day]T[hour repr:24]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]"
        );
        let unix = format_description!("[unix_timestamp]");

        html! {
            <div class="flex flex-col items-center justify-around full">
                <div class="text-3xl">{self.time.format(date).unwrap()}</div>
                <div class="text-7xl">{self.time.format(time).unwrap()}</div>
                <div class="flex items-center justify-around self-stretch text-gray-400">
                    <div data-clock-target="iso">{self.time.format(iso).unwrap()}</div>
                    <div data-clock-target="unix">{self.time.format(unix).unwrap()}</div>
                </div>
            </div>
        }
    }
}
