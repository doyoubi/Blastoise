use std::fmt::{Display, Debug};
use std::option::Option::None;
use std::result::Result::Ok;
use std::ptr::{write, read};
use libc::malloc;
use ::parser::lexer::TokenIter;
use ::parser::compile_error::ErrorList;
use ::parser::common::exp_list_to_string;
use ::utils::pointer::{write_string, read_string};
use ::store::buffer::DataPtr;


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

macro_rules! gen_plan_helper {
    ($input_str:expr, $manager:expr) => ({
        use ::exec::gen_plan::gen_table_set;
        use ::parser::common::Statement;
        use ::parser::sem_check::check_sem;
        use ::exec::gen_plan::gen_plan;

        let tokens = gen_token!($input_str);
        let stmt = Statement::parse(&mut tokens.iter());
        let mut stmt = extract!(stmt, Ok(stmt), stmt);
        let table_set = gen_table_set(&stmt, &$manager);
        assert_pattern!(check_sem(&mut stmt, &table_set), Ok(()));
        gen_plan(stmt, $manager)
    })
}

macro_rules! gen_parse_result {
    ($class:ident :: $parse_func:ident, $input_str:expr) => ({
        let tokens = gen_token!($input_str);
        extract!($class::$parse_func(&mut tokens.iter()), Ok(result), result)
    })
}

// test code in ::utils
#[test]
fn test_raw_str_convert() {
    unsafe{
        let p : DataPtr = malloc(3);
        let s = "ab".to_string();
        write_string(p, &s, 3);
        assert_eq!(read::<u8>(p as *const u8), 97);
        assert_eq!(read::<u8>((p as *const u8).offset(1)), 98);
        assert_eq!(read_string(p, 2), "ab");
        assert_eq!(read_string(p, 3), "ab");
    }
    unsafe{
        let p : DataPtr = malloc(3);
        let s = "abc".to_string();
        write_string(p, &s, 3);
        assert_eq!(read::<u8>(p as *const u8), 97);
        assert_eq!(read::<u8>((p as *const u8).offset(1)), 98);
        assert_eq!(read::<u8>((p as *const u8).offset(2)), 99);
        assert_eq!(read_string(p, 3), "abc");
    }
}
