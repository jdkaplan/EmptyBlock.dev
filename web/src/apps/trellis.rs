use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub layout: Layout,
    pub secrets: Secrets,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub tiles: Vec<Tile>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Secrets {
    pub open_weather: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tile {
    pub id: Uuid,
    pub data: Data,
    // TODO: height/width hints
    // TODO: title
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Data {
    Clock, // TODO: option to show a specific time zone
    Weather(Weather),
    Note(Note),
    Counter(Counter),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weather {
    pub location_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counter {
    pub value: i64,
}

const STARTER_NOTE: &str = r#"Welcome to Trellis!

This is _very_ much a work-in-progress, and there are _definitely_ major bugs. (For example, changes to this text box don't save yet!)

If you think this is cool, have an idea to share, or want to watch development, find the project link on the About page!"#;

impl Config {
    pub fn starter() -> Self {
        Self {
            secrets: Secrets { open_weather: None },
            layout: Layout {
                tiles: vec![
                    Tile {
                        id: Uuid::new_v4(),
                        data: Data::Clock,
                    },
                    Tile {
                        id: Uuid::new_v4(),
                        data: Data::Weather(Weather { location_id: None }),
                    },
                    Tile {
                        id: Uuid::new_v4(),
                        data: Data::Note(Note {
                            text: String::from(STARTER_NOTE),
                        }),
                    },
                ],
            },
        }
    }
}
