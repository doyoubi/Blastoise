use std::fmt::{Display, Debug};
use std::option::Option::None;
use std::result::Result::Ok;
use ::parser::lexer::TokenIter;
use ::parser::compile_error::ErrorList;
use ::parser::common::exp_list_to_string;


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
            other => panic!("assert pattern fail, pattern not matched, found {:?}", other),
        }
    )
}

pub fn test_by_display_str<Res : Display + Debug>(
        input_str : &str,
        token_num : usize,
        parse_func : fn(&mut TokenIter) -> Result<Res, ErrorList>,
        debug_str : &str) {
    let tokens = gen_token!(input_str);
    assert_eq!(tokens.len(), token_num);
    let mut it = tokens.iter();
    let res = parse_func(&mut it);
    let res = extract!(res, Ok(res), res);
    assert_eq!(format!("{}", res), debug_str);
    assert_pattern!(it.next(), None);
}

pub fn test_by_list_to_str<Res : Display + Debug>(
        input_str : &str,
        token_num : usize,
        parse_func : fn(&mut TokenIter) -> Result<Vec<Res>, ErrorList>,
        debug_str : &str) {
    let tokens = gen_token!(input_str);
    assert_eq!(tokens.len(), token_num);
    let mut it = tokens.iter();
    let res = parse_func(&mut it);
    let res = extract!(res, Ok(res), res);
    assert_eq!(exp_list_to_string(&res), debug_str);
    assert_pattern!(it.next(), None);
}

pub fn remove_blanks(s : &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '\n' | ' ' | '\t' => continue,
            _ => result.push(c),
        };
    }
    result
}
