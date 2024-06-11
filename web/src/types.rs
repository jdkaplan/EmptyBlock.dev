macro_rules! type_ {
    ($name:ident) => {
        pub mod $name;
        pub use $name::*;
    };
}

type_!(csrf_token);
type_!(session);
