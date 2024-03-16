#![allow(unused_imports)]

macro_rules! component {
    ($component:ident) => {
        pub mod $component;
        pub use $component::*;
    };
}

component!(block);
component!(board);
component!(board_editor);
component!(clock);
component!(counter);
component!(error);
component!(footer);
component!(grid);
component!(header);
component!(note);
component!(simulation);
component!(simulation_editor);
component!(trellis_config_context);
component!(weather);
