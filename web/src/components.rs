macro_rules! component {
    ($component:ident) => {
        pub mod $component;
        pub use $component::*;
    };
}

component!(block);
component!(editor);
component!(grid);
component!(simulation);
