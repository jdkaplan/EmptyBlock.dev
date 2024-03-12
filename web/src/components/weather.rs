use gloo::net::http::Request;
use gloo::timers::callback::Interval;
use once_cell::sync::Lazy;
use serde::Deserialize;
use time::OffsetDateTime;
use url::Url;
use yew::prelude::*;

static OWM_WEATHER_ENDPOINT: Lazy<Url> = Lazy::new(|| {
    Url::parse("https://api.openweathermap.org/data/2.5/weather").expect("static URL")
});
static OWM_ICON_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://openweathermap.org/img/w/").expect("static URL"));
static OWM_CITY_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://openweathermap.org/city/").expect("static URL"));

#[derive(Properties, PartialEq, Clone)]
pub struct WeatherProps {
    #[prop_or_default]
    pub location_id: AttrValue,
    #[prop_or_default]
    pub owm_api_key: AttrValue,
}

pub struct Weather {
    location: Location,
    last_updated_at: Option<OffsetDateTime>,
    error: Option<WeatherError>,

    _interval: Interval,
}

#[derive(Clone, Default)]
struct Location {
    name: String,
    icon: String,
    description: String,
    temperature: String,
}

pub enum WeatherMsg {
    Fetch,
    Render(Result<OwmResponse, WeatherError>),
}

impl Component for Weather {
    type Message = WeatherMsg;
    type Properties = WeatherProps;

    fn create(ctx: &Context<Self>) -> Self {
        // Immediately trigger the first fetch ...
        ctx.link().send_message(WeatherMsg::Fetch);

        // ... and then set up a timer for the rest of them.
        let interval = {
            let link = ctx.link().clone();
            Interval::new(60_000, move || link.send_message(WeatherMsg::Fetch))
        };

        Self {
            location: Location::default(),
            last_updated_at: None,
            error: None,

            _interval: interval,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            WeatherMsg::Fetch => {
                let props = ctx.props().clone();
                ctx.link().send_future(async {
                    let res = fetch_weather(props).await;
                    WeatherMsg::Render(res)
                });

                // TODO: Show a "currently fetching" indicator
                false
            }

            WeatherMsg::Render(res) => {
                let data = match res {
                    Ok(data) => data,
                    Err(err) => {
                        self.error = Some(err);
                        return true;
                    }
                };

                // The weather comes back as an array. This is probably one item per
                // weather station, so taking the first one should be fine for now.
                //
                // TODO: Figure out how to handle this list better.
                let weather = &data.weather[0];
                let icon_url = OWM_ICON_URL
                    .join(&format!("{}.png", weather.icon))
                    .expect("icon name is always a valid path component");

                self.location = Location {
                    name: data.name,
                    temperature: format!("{:.0}℉", data.main.temp),
                    description: weather.description.clone(),
                    icon: icon_url.to_string(),
                };
                self.error = None;

                let now = OffsetDateTime::now_local().unwrap();
                self.last_updated_at = Some(now);

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.last_updated_at {
            None => self.view_loading(),
            Some(_) => self.view_weather(ctx.props()),
        }
    }
}

impl Weather {
    fn view_loading(&self) -> Html {
        html! {
            <div class="flex flex-col items-center justify-around w-full h-full">
                <div class="text-xl text-red-500">
                    {self.error.as_ref().map_or(String::new(), |e| e.to_string())}
                </div>
                <p>{"Loading..."}</p>
            </div>
        }
    }

    fn view_weather(&self, props: &WeatherProps) -> Html {
        use time::macros::format_description;

        let time = format_description!("[hour repr:24]:[minute]:[second]");

        let last_updated = self
            .last_updated_at
            .map_or(String::from("never"), |t| t.format(time).unwrap());

        let owm_url = OWM_CITY_URL
            .join(&props.location_id)
            .expect("location ID is always a valid path segment")
            .to_string();

        let location = self.location.clone();
        let name = location.name;
        let icon = location.icon;
        let description = location.description;
        let temperature = location.temperature;

        html! {
            <div class="flex flex-col items-center justify-around w-full h-full">
                <span class="text-3xl">{name}</span>
                <img src={icon} alt={description.clone()} />
                <span class="text-2xl">{description}</span>
                <div class="text-2xl">{temperature}</div>
                <div class="text-xl text-red-500">
                    {self.error.as_ref().map_or(String::new(), |e| e.to_string())}
                </div>
                <div class="flex items-center justify-around self-stretch text-gray-400">
                    <a href={owm_url}>{"OpenWeatherMap"}</a>
                    <div>
                        {format!("Last updated at: {}", last_updated)}
                    </div>
                </div>
            </div>
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct OwmResponse {
    weather: Vec<OwmWeather>,
    main: OwmMain,
    name: String,
}

#[derive(Deserialize, Clone)]
pub struct OwmWeather {
    description: String,
    icon: String,
}

#[derive(Deserialize, Clone)]
pub struct OwmMain {
    temp: f64,
}

#[derive(thiserror::Error, Debug)]
pub enum WeatherError {
    #[error("missing OpenWeatherMap API key")]
    MissingApiKey,

    #[error("missing location ID")]
    MissingLocation,

    #[error(transparent)]
    Fetch(#[from] gloo::net::Error),
}

async fn fetch_weather(props: WeatherProps) -> Result<OwmResponse, WeatherError> {
    if props.owm_api_key.is_empty() {
        return Err(WeatherError::MissingApiKey);
    }
    if props.location_id.is_empty() {
        return Err(WeatherError::MissingLocation);
    }

    let res = Request::get(OWM_WEATHER_ENDPOINT.as_str())
        .query([
            ("id", props.location_id.as_str()),
            ("appid", props.owm_api_key.as_str()),
            ("units", "imperial"),
        ])
        .send()
        .await?;

    let weather = res.json().await?;
    Ok(weather)
}
