macro_rules! extract {
    ($expression:expr, $pattern:pat, $returned_value:expr) => (
        match $expression {
            $pattern => $returned_value,
            _ => panic!("extract error, unexpected pattern: {:?}", $expression),
        }
    )
}

macro_rules! is_match {
    ($expression:expr, $pattern:pat) => (
        match $expression {
            $pattern => true,
            _ => false,
        }
    );
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

macro_rules! lock_unwrap {
    ($result:expr) => ({
        use std::result::Result::{Ok, Err};
        match $result {
            Ok(guard) => guard,
            Err(err) => panic!("lock error accur {:?}", err),
        }
    })
}

macro_rules! check_ok {
    ($expression:expr) => ({
        use std::result::Result::{Ok, Err};
        match $expression {
            Ok(v) => (v),
            Err(err) => panic!("runtime error: {:?}", err),
        }
    })
}
