use std::rc::Rc;

use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage};
use uuid::Uuid;
use yew::prelude::*;

use crate::apps::trellis::{Config, Data};

const LOCAL_STORAGE_KEY: &str = "trellis.config";

pub type TrellisConfigContext = UseReducerHandle<TrellisConfig>;

#[derive(Properties, PartialEq, Debug)]
pub struct TrellisConfigProviderProps {
    #[prop_or_default]
    pub children: Html,
}

#[function_component]
pub fn TrellisConfigProvider(props: &TrellisConfigProviderProps) -> Html {
    let config = use_reducer(TrellisConfig::load);

    html! {
        <ContextProvider<TrellisConfigContext> context={config}>
            { props.children.clone() }
        </ContextProvider<TrellisConfigContext>>
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TrellisConfig {
    pub inner: Result<Config, String>,
}

impl TrellisConfig {
    fn load() -> Self {
        let inner = match LocalStorage::get(LOCAL_STORAGE_KEY) {
            Ok(config) => Ok(config),
            Err(err @ StorageError::KeyNotFound(_)) => {
                tracing::info!({ ?err }, "No Trellis config found. Using starter config");
                Ok(Config::starter())
            }
            Err(err) => {
                let value = LocalStorage::raw().get_item(LOCAL_STORAGE_KEY);
                tracing::error!({ ?err, ?value }, "Could not parse Trellis config");
                Err(err.to_string())
            }
        };

        Self { inner }
    }
}

pub enum TrellisConfigAction {
    Save(Config),
    Update { id: Uuid, data: Data },
}

impl Reducible for TrellisConfig {
    type Action = TrellisConfigAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            TrellisConfigAction::Save(config) => {
                // TODO(users): Sync LocalStorage with the DB
                LocalStorage::set(LOCAL_STORAGE_KEY, &config).unwrap();
                Rc::new(Self { inner: Ok(config) })
            }

            TrellisConfigAction::Update { id, data } => {
                tracing::debug!({ ?id, ?data}, "Trellis Config update");

                let config = self.inner.as_ref().expect("can only update valid config");
                let mut config = config.clone();

                for tile in &mut config.layout.tiles {
                    if tile.id == id {
                        tile.data = data.clone();
                    }
                }

                // TODO(users): Sync LocalStorage with the DB
                LocalStorage::set(LOCAL_STORAGE_KEY, &config).unwrap();
                Rc::new(Self { inner: Ok(config) })
            }
        }
    }
}
