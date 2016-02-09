macro_rules! extract {
    ($expression:expr, $pattern:pat, $returned_value:expr) => (
        match $expression {
            $pattern => $returned_value,
            _ => panic!("unexpected pattern: {:?}", $expression),
        }
    )
}

macro_rules! impl_debug_from_display {
    ($type_name:ident) => (
        impl $type_name {
            use std::fmt;
            fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self)
            }
        }
    );
}
