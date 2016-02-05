macro_rules! gen_token {
    ($input_str:expr) => ({
            let line = ::parser::lexer::TokenLine::parse($input_str);
            assert!(line.errors.is_empty());
            line.tokens.clone()
        });
}

macro_rules! assert_pattern {
    ($expression:expr, $pattern:pat) => (
        match $expression {
            $pattern => (),
            other => panic!("pattern not matched, found {:?}", other),
        }
    )
}
