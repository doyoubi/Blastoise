macro_rules! extract {
    ($expression:expr, $pattern:pat, $returned_value:expr) => (
        match $expression {
            $pattern => $returned_value,
            _ => unreachable!(),
        }
    )
}
