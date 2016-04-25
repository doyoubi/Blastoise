use std::option::Option::None;
use std::result::Result::{Ok, Err};
use std::vec::Vec;
use std::fmt::Debug;
use ::parser::lexer::TokenIter;
use ::parser::condition::{ArithExpr, ParseArithResult, CmpOperantExpr, ParseCondResult, ConditionExpr};
use ::parser::common::{ValueType, ValueExpr};
use ::parser::compile_error::{CompileErrorType, ErrorList};
use ::parser::attribute::AttributeExpr;


macro_rules! test_literal {
    ($input_str:expr, $value:expr, $token_type:expr, $expr_type:ident :: $parse_func:ident) => ({
        let tokens = gen_token!($input_str);
        assert_eq!(tokens.len(), 1);
        let mut it = tokens.iter();
        let arith_exp = $expr_type::$parse_func(&mut it);
        assert_pattern!(arith_exp, Ok(_));
        let arith_exp = arith_exp.unwrap();
        assert_eq!(arith_exp.to_string(),
            format!("{:?}({})", $token_type, $value));
        let (value, value_type) = extract!(
            arith_exp, $expr_type::Value(ValueExpr{ value, value_type }), (value, value_type));
        assert_eq!(value, $value);
        assert_eq!(value_type, $token_type);
        assert_pattern!(it.next(), None);
    });
}

type ParseArithFun = fn(&mut TokenIter) -> ParseArithResult;

fn test_invalid_tokens<Exp : Debug>(parse_func : fn(&mut TokenIter) -> Result<Exp, ErrorList>,
        input_str : &str, token_num : usize, error_type : CompileErrorType) {
    let tokens = gen_token!(input_str);
    assert_eq!(tokens.len(), token_num);
    let mut it = tokens.iter();
    let attr_exp = parse_func(&mut it);
    assert_pattern!(attr_exp, Err(_));
    let ref errs = attr_exp.unwrap_err();
    let ref err = errs[0];
    assert_eq!(err.error_type, error_type);
}

fn test_single_attribute_name(parse_func : ParseArithFun) {
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
    test_literal!("233", "233", ValueType::Integer, ArithExpr::parse_arith_operant);
    test_literal!("233.666", "233.666", ValueType::Float, ArithExpr::parse_arith_operant);
    test_single_attribute_name(ArithExpr::parse_arith_operant);
    test_invalid_tokens(ArithExpr::parse_arith_operant, "or", 1, CompileErrorType::ParserUnExpectedTokenType);
}

fn test_minus_expr(parse_func : ParseArithFun) {
    let tokens = gen_token!("-1");
    assert_eq!(tokens.len(), 2);
    let mut it = tokens.iter();
    let minus_exp = parse_func(&mut it);
    assert_pattern!(minus_exp, Ok(..));
    let minus_exp = minus_exp.unwrap();
    assert_eq!(minus_exp.to_string(), "(- Integer(1))");
    let inner_exp = extract!(minus_exp, ArithExpr::MinusExpr{operant}, operant);
    let (value, value_type) = extract!(
        *inner_exp, ArithExpr::Value(ValueExpr{ref value, value_type}), (value.clone(), value_type));
    assert_eq!(value, "1");
    assert_pattern!(value_type, ValueType::Integer);
    assert_pattern!(it.next(), None);
}

fn test_plus_expr(parse_func : ParseArithFun) {
    let tokens = gen_token!("+1");
    assert_eq!(tokens.len(), 2);
    let mut it = tokens.iter();
    let value_exp = parse_func(&mut it);
    assert_pattern!(value_exp, Ok(..));
    let value_exp = value_exp.unwrap();
    assert_eq!(value_exp.to_string(), "Integer(1)");
    let (value, value_type) = extract!(
        value_exp, ArithExpr::Value(ValueExpr{ref value, value_type}), (value.clone(), value_type));
    assert_eq!(value, "1".to_string());
    assert_pattern!(value_type, ValueType::Integer);
    assert_pattern!(it.next(), None);
}

fn test_bracket(parse_func : ParseArithFun) {
    let tokens = gen_token!("(1)");
    assert_eq!(tokens.len(), 3);
    let mut it = tokens.iter();
    let value_exp = parse_func(&mut it);
    assert_pattern!(value_exp, Ok(..));
    let value_exp = value_exp.unwrap();
    assert_eq!(value_exp.to_string(), "Integer(1)");
    let (value, value_type) = extract!(
        value_exp, ArithExpr::Value(ValueExpr{ref value, value_type}), (value.clone(), value_type));
    assert_eq!(value, "1".to_string());
    assert_pattern!(value_type, ValueType::Integer);
    assert_pattern!(it.next(), None);
}

#[test]
fn test_arith_parse_primitive() {
    test_minus_expr(ArithExpr::parse_primitive);
    test_plus_expr(ArithExpr::parse_primitive);
    test_bracket(ArithExpr::parse_primitive);
    test_single_attribute_name(ArithExpr::parse_primitive);
}

fn test_parse_second_binary(parse_func : ParseArithFun, ops : Vec<&str>) {
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

fn test_parse_longer_second_binary(parse_func : ParseArithFun, ops : Vec<&str>) {
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
    test_invalid_tokens(ArithExpr::parse_second_binary, "*", 1, CompileErrorType::ParserUnExpectedTokenType);

    test_parse_second_binary(ArithExpr::parse_first_binary, vec!["+", "-"]);
    test_parse_longer_second_binary(ArithExpr::parse_first_binary, vec!["+", "-"]);
    test_invalid_tokens(ArithExpr::parse_second_binary, "*", 1, CompileErrorType::ParserUnExpectedTokenType);
}

#[test]
fn test_parse_complex_arith_exp() {
    let tokens = gen_token!("1 + attr_name* table_name.attr_name - (2 + +3)");
    assert_eq!(tokens.len(), 14);
    let mut it = tokens.iter();
    let exp = ArithExpr::parse(&mut it);
    assert_pattern!(exp, Ok(..));
    let bin_exp = exp.unwrap();
    assert_eq!(bin_exp.to_string(),
        "((Integer(1) + (attr_name * (table_name.attr_name))) - (Integer(2) + Integer(3)))");
    assert_pattern!(it.next(), None);
}

#[test]
fn test_cmp_operant_parse() {
    test_literal!("\"string\"", "string", ValueType::String, CmpOperantExpr::parse);
    test_literal!("null", "null", ValueType::Null, CmpOperantExpr::parse);
    test_invalid_tokens(CmpOperantExpr::parse, "*", 1, CompileErrorType::ParserUnExpectedTokenType);
}

type ParseCondFun = fn(&mut TokenIter) -> ParseCondResult;

fn test_parse_cmp(parse_func : ParseCondFun) {
    for op in &["<", ">", "<=", ">=", "=", "!=", "is", "is not"] {
        let input_str = format!("1 {} 2", op);
        let tokens = gen_token!(&input_str);
        assert_eq!(tokens.len(), 3);
        let mut it = tokens.iter();
        let exp = parse_func(&mut it);
        assert_pattern!(exp, Ok(..));
        let exp = exp.unwrap();
        assert_eq!(exp.to_string(),
            format!("(Integer(1) {} Integer(2))", op));
        assert_pattern!(it.next(), None);
    }
}

#[test]
fn test_cmp() {
    test_parse_cmp(ConditionExpr::parse_cmp);
    test_invalid_tokens(ConditionExpr::parse_cmp, "1 + 2", 3, CompileErrorType::ParserNoMoreToken)
}

fn test_parse_not_cond(parse_func : ParseCondFun) {
    let tokens = gen_token!("not 1 > 2");
    assert_eq!(tokens.len(), 4);
    let mut it = tokens.iter();
    let exp = parse_func(&mut it);
    assert_pattern!(exp, Ok(..));
    let exp = exp.unwrap();
    assert_eq!(exp.to_string(), "(not (Integer(1) > Integer(2)))");
    assert_pattern!(it.next(), None);
}

fn test_cond_parse_braket(parse_func : ParseCondFun) {
    let tokens = gen_token!("(1 > 2)");
    assert_eq!(tokens.len(), 5);
    let mut it = tokens.iter();
    let exp = parse_func(&mut it);
    assert_pattern!(exp, Ok(..));
    let exp = exp.unwrap();
    assert_eq!(exp.to_string(), "(Integer(1) > Integer(2))");
    assert_pattern!(it.next(), None);
}

#[test]
fn test_cond_parse_primitive() {
    test_parse_not_cond(ConditionExpr::parse_primitive);
    test_cond_parse_braket(ConditionExpr::parse_primitive);
    test_parse_cmp(ConditionExpr::parse_primitive);
    test_invalid_tokens(ConditionExpr::parse_primitive, "1 + 2", 3, CompileErrorType::ParserNoMoreToken)
}

fn test_or_and(parse_func : ParseCondFun) {
    let tokens = gen_token!("1 > 2 or 3 < 4 and 5 = 6 or 7 >= 8");
    assert_eq!(tokens.len(), 15);
    let mut it = tokens.iter();
    let exp = parse_func(&mut it);
    assert_pattern!(exp, Ok(..));
    let exp = exp.unwrap();
    assert_eq!(exp.to_string(),
        "(((Integer(1) > Integer(2)) or ((Integer(3) < Integer(4)) and (Integer(5) = Integer(6)))) or (Integer(7) >= Integer(8)))");
    assert_pattern!(it.next(), None);
}

fn test_or_and_with_bracket(parse_func : ParseCondFun) {
    let tokens = gen_token!("1 > 2 or (3 < 4 or 7 >= 8)");
    assert_eq!(tokens.len(), 13);
    let mut it = tokens.iter();
    let exp = parse_func(&mut it);
    assert_pattern!(exp, Ok(..));
    let exp = exp.unwrap();
    assert_eq!(exp.to_string(),
        "((Integer(1) > Integer(2)) or ((Integer(3) < Integer(4)) or (Integer(7) >= Integer(8))))");
    assert_pattern!(it.next(), None);
}

#[test]
fn test_binary_cond_exp() {
    test_or_and(ConditionExpr::parse_or);
    test_or_and_with_bracket(ConditionExpr::parse_or);
    test_invalid_tokens(ConditionExpr::parse_or, "1 + 2", 3, CompileErrorType::ParserNoMoreToken)
}

#[test]
fn test_complex_cond_expr() {
    {
        let tokens = gen_token!(
            "person.money is not null and 1/2 > 3 or 4%(5-6) > 7 and not employee = \"doyoubi\"");
        assert_eq!(tokens.len(), 26);
        let mut it = tokens.iter();
        let exp = ConditionExpr::parse(&mut it);
        assert_pattern!(exp, Ok(..));
        let exp = exp.unwrap();
        let part1 = "((person.money) is not Null(null))";
        let part2 = "((Integer(1) / Integer(2)) > Integer(3))";
        let part3 = "((Integer(4) % (Integer(5) - Integer(6))) > Integer(7))";
        let part4 = "(not (employee = String(doyoubi)))";
        let result = format!("(({} and {}) or ({} and {}))", part1, part2, part3, part4);
        assert_eq!(exp.to_string(), result);
        assert_pattern!(it.next(), None);
    }
    {
        let tokens = gen_token!("dept.number > 1");
        assert_eq!(tokens.len(), 5);
        let mut it = tokens.iter();
        let exp = ConditionExpr::parse(&mut it);
        assert_pattern!(exp, Ok(..));
        let exp = exp.unwrap();
        assert_eq!(exp.to_string(), "((dept.number) > Integer(1))");
        assert_pattern!(it.next(), None);
    }
    {
        let tokens = gen_token!("(MATH_SCORE + ENGLISH_SCORE) / 2 >= 80");
        assert_eq!(tokens.len(), 9);
        let mut it = tokens.iter();
        let exp = ConditionExpr::parse(&mut it);
        assert_pattern!(exp, Ok(..));
        let exp = exp.unwrap();
        assert_eq!(exp.to_string(), "(((MATH_SCORE + ENGLISH_SCORE) / Integer(2)) >= Integer(80))");
        assert_pattern!(it.next(), None);
    }
}
