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
component!(grid);
component!(note);
component!(simulation);
component!(simulation_editor);
component!(weather);
