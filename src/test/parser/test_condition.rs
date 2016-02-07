use std::option::Option::None;
use std::result::Result::{Ok, Err};
use std::vec::Vec;
use ::parser::lexer::TokenIter;
use ::parser::condition::{ArithExpr, ParseArithResult};
use ::parser::common::ValueType;
use ::parser::compile_error::CompileErrorType;
use ::parser::attribute::AttributeExpr;


macro_rules! test_literal {
    ($input_str:expr, $value:expr, $token_type:expr) => ({
        let tokens = gen_token!($input_str);
        assert_eq!(tokens.len(), 1);
        let mut it = tokens.iter();
        let arith_exp = ArithExpr::parse_arith_operant(&mut it);
        assert_pattern!(arith_exp, Ok(_));
        let arith_exp = arith_exp.unwrap();
        assert_eq!(arith_exp.to_string(),
            format!("{:?}({})", $token_type, $value));
        let (value, value_type) = extract!(
            arith_exp, ArithExpr::ValueExpr{ value, value_type }, (value, value_type));
        assert_eq!(value, $value);
        assert_eq!(value_type, $token_type);
        assert_pattern!(it.next(), None);
    });
}

type ParseFun = fn(&mut TokenIter) -> ParseArithResult;

fn test_invalid_tokens(parse_func : ParseFun, input_str : &str) {
    let tokens = gen_token!(input_str);
    assert_eq!(tokens.len(), 1);
    let mut it = tokens.iter();
    assert_eq!(it.len(), 1);
    let attr_exp = parse_func(&mut it);
    assert_pattern!(attr_exp, Err(_));
    let ref errs = attr_exp.unwrap_err();
    let ref err = errs[0];
    assert_eq!(err.error_type, CompileErrorType::ParserUnExpectedTokenType);
}

fn test_single_attribute_name(parse_func : ParseFun) {
    let tokens = gen_token!("attribute_name");
    assert_eq!(tokens.len(), 1);
    let mut it = tokens.iter();
    let attr_exp = parse_func(&mut it);
    assert_pattern!(attr_exp, Ok(..));
    let attr_exp = attr_exp.unwrap();
    assert_eq!(attr_exp.to_string(), "attribute_name");
    let (table, attr) = extract!(attr_exp, ArithExpr::Attr(AttributeExpr::TableAttr{ table, attr }), (table, attr));
    assert!(!table.is_some());
    assert_eq!(attr, "attribute_name");
    assert_pattern!(it.next(), None);
}

#[test]
fn test_parse_arith_operant() {
    test_literal!("233", "233", ValueType::Integer);
    test_literal!("233.666", "233.666", ValueType::Float);
    test_single_attribute_name(ArithExpr::parse_arith_operant);
    test_invalid_tokens(ArithExpr::parse_arith_operant, "or");
}

fn test_minus_expr(parse_func : ParseFun) {
    let tokens = gen_token!("-1");
    assert_eq!(tokens.len(), 2);
    let mut it = tokens.iter();
    let minus_exp = parse_func(&mut it);
    assert_pattern!(minus_exp, Ok(..));
    let minus_exp = minus_exp.unwrap();
    assert_eq!(minus_exp.to_string(), "(- Integer(1))");
    let inner_exp = extract!(minus_exp, ArithExpr::MinusExpr{operant}, operant);
    let (value, value_type) = extract!(
        *inner_exp, ArithExpr::ValueExpr{ref value, value_type}, (value.clone(), value_type));
    assert_eq!(value, "1");
    assert_pattern!(value_type, ValueType::Integer);
    assert_pattern!(it.next(), None);
}

fn test_plus_expr(parse_func : ParseFun) {
    let tokens = gen_token!("+1");
    assert_eq!(tokens.len(), 2);
    let mut it = tokens.iter();
    let value_exp = parse_func(&mut it);
    assert_pattern!(value_exp, Ok(..));
    let value_exp = value_exp.unwrap();
    assert_eq!(value_exp.to_string(), "Integer(1)");
    let (value, value_type) = extract!(
        value_exp, ArithExpr::ValueExpr{ref value, value_type}, (value.clone(), value_type));
    assert_eq!(value, "1".to_string());
    assert_pattern!(value_type, ValueType::Integer);
    assert_pattern!(it.next(), None);
}

fn test_bracket(parse_func : ParseFun) {
    let tokens = gen_token!("(1)");
    assert_eq!(tokens.len(), 3);
    let mut it = tokens.iter();
    let value_exp = parse_func(&mut it);
    assert_pattern!(value_exp, Ok(..));
    let value_exp = value_exp.unwrap();
    assert_eq!(value_exp.to_string(), "Integer(1)");
    let (value, value_type) = extract!(
        value_exp, ArithExpr::ValueExpr{ref value, value_type}, (value.clone(), value_type));
    assert_eq!(value, "1".to_string());
    assert_pattern!(value_type, ValueType::Integer);
    assert_pattern!(it.next(), None);
}

#[test]
fn test_parse_primitive() {
    test_minus_expr(ArithExpr::parse_primitive);
    test_plus_expr(ArithExpr::parse_primitive);
    test_bracket(ArithExpr::parse_primitive);
    test_single_attribute_name(ArithExpr::parse_primitive);
}

fn test_parse_second_binary(parse_func : ParseFun, ops : Vec<&str>) {
    for op in &ops {
        let input_str = format!("1 {}233", op);
        let tokens = gen_token!(&input_str);
        assert_eq!(tokens.len(), 3);
        let mut it = tokens.iter();
        let bin_exp = parse_func(&mut it);
        assert_pattern!(bin_exp, Ok(..));
        let bin_exp = bin_exp.unwrap();
        assert_eq!(bin_exp.to_string(),
            format!("(Integer(1) {} Integer(233))", op));
        assert_pattern!(it.next(), None);
    }
}

fn test_parse_longer_second_binary(parse_func : ParseFun, ops : Vec<&str>) {
    for op in &ops {
        let input_str = format!("1 {}233{} 1", op, op);
        let tokens = gen_token!(&input_str);
        assert_eq!(tokens.len(), 5);
        let mut it = tokens.iter();
        let bin_exp = parse_func(&mut it);
        assert_pattern!(bin_exp, Ok(..));
        let bin_exp = bin_exp.unwrap();
        assert_eq!(bin_exp.to_string(),
            format!("((Integer(1) {} Integer(233)) {} Integer(1))", op, op));
        assert_pattern!(it.next(), None);
    }
}

#[test]
fn test_parse_binary() {
    test_parse_second_binary(ArithExpr::parse_second_binary, vec!["*", "/", "%"]);
    test_parse_longer_second_binary(ArithExpr::parse_second_binary, vec!["*", "/", "%"]);
    test_invalid_tokens(ArithExpr::parse_second_binary, "*");

    test_parse_second_binary(ArithExpr::parse_first_binary, vec!["+", "-"]);
    test_parse_longer_second_binary(ArithExpr::parse_first_binary, vec!["+", "-"]);
    test_invalid_tokens(ArithExpr::parse_second_binary, "*");
}

#[test]
fn test_parse_complex_arith_exp() {
    let tokens = gen_token!("1 + attr_name* table_name.attr_name - (2 + 3)");
    assert_eq!(tokens.len(), 13);
    let mut it = tokens.iter();
    let exp = ArithExpr::parse(&mut it);
    assert_pattern!(exp, Ok(..));
    let bin_exp = exp.unwrap();
    assert_eq!(bin_exp.to_string(),
        "((Integer(1) + (attr_name * (table_name.attr_name))) - (Integer(2) + Integer(3)))");
    assert_pattern!(it.next(), None);
}
