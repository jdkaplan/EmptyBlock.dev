macro_rules! page {
    ($page:ident) => {
        pub mod $page;
        pub use $page::*;
    };
}

page!(home);
page!(mosaic);
page!(trellis);
page!(trellis_config);
