macro_rules! page {
    ($page:ident) => {
        pub mod $page;
        pub use $page::*;
    };
}

page!(home);
page!(mosaic);
page!(not_found);
page!(trellis);
page!(trellis_config);
