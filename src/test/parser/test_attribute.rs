use std::option::Option;
use std::option::Option::{Some, None};
use std::result::Result::Ok;
use ::parser::lexer::TokenIter;
use ::parser::attribute::{AttributeExpr, ParseAttrResult, AttributeList};
use ::parser::compile_error::CompileErrorType;


type ParseFun = fn(&mut TokenIter) -> ParseAttrResult;

fn test_single_attribute_name(parse_func : ParseFun) {
    let tokens = gen_token!("attribute_name");
    assert_eq!(tokens.len(), 1);
    let mut it = tokens.iter();
    let attr_exp = parse_func(&mut it);
    assert_pattern!(attr_exp, Ok(_));
    let attr_exp = attr_exp.unwrap();
    assert_eq!(attr_exp.to_string(), "attribute_name");
    let (table, attr) = extract!(attr_exp, AttributeExpr::TableAttr{ table, attr }, (table, attr));
    assert!(!table.is_some());
    assert_eq!(attr, "attribute_name".to_string());
    assert_pattern!(it.next(), None);
}

fn test_table_attribute(parse_func : ParseFun) {
    let tokens = gen_token!("table_name.attribute_name");
    assert_eq!(tokens.len(), 3);
    let mut it = tokens.iter();
    let attr_exp = parse_func(&mut it);
    assert_pattern!(attr_exp, Ok(_));
    let attr_exp = attr_exp.unwrap();
    assert_eq!(attr_exp.to_string(), "(table_name.attribute_name)");
    let (table, attr) = extract!(attr_exp, AttributeExpr::TableAttr{ table, attr }, (table, attr));
    assert_eq!(table, Some("table_name".to_string()));
    assert_eq!(attr, "attribute_name".to_string());
    assert_pattern!(it.next(), None);
}

fn test_invalid_tokens(parse_func : ParseFun) {
    let tokens = gen_token!("1");
    assert_eq!(tokens.len(), 1);
    let mut it = tokens.iter();
    assert_eq!(it.len(), 1);
    let attr_exp = parse_func(&mut it);
    assert_pattern!(attr_exp, Err(_));
    let ref errs = attr_exp.unwrap_err();
    let ref err = errs[0];
    assert_eq!(err.error_type, CompileErrorType::ParserUnExpectedTokenType);
}

#[test]
fn test_parse_table_attr() {
    test_single_attribute_name(AttributeExpr::parse_table_attr);
    test_table_attribute(AttributeExpr::parse_table_attr);
    test_invalid_tokens(AttributeExpr::parse_table_attr);
}

fn test_aggre_func_with_table_name(parse_func : ParseFun) {
    let tokens = gen_token!("sum(table_name.attribute_name)");
    assert_eq!(tokens.len(), 6);
    let mut it = tokens.iter();
    let func_exp = parse_func(&mut it);
    assert_pattern!(func_exp, Ok(..));
    let func_exp = func_exp.unwrap();
    assert_eq!(func_exp.to_string(), "sum(table_name.attribute_name)");
    let (func, table, attr) = extract!(
        func_exp, AttributeExpr::AggreFuncCall{ func, table, attr }, (func, table, attr));
    assert_eq!(func, "sum".to_string());
    assert_eq!(table, Some("table_name".to_string()));
    assert_eq!(attr, "attribute_name".to_string());
    assert_pattern!(it.next(), None);
}

fn test_aggre_func_with_single_attr(parse_func : ParseFun) {
    let tokens = gen_token!("sum(attribute_name)");
    assert_eq!(tokens.len(), 4);
    let mut it = tokens.iter();
    let func_exp = parse_func(&mut it);
    assert_pattern!(func_exp, Ok(..));
    let func_exp = func_exp.unwrap();
    assert_eq!(func_exp.to_string(), "sum(attribute_name)");
    let (func, table, attr) = extract!(
        func_exp, AttributeExpr::AggreFuncCall{ func, table, attr }, (func, table, attr));
    assert_eq!(func, "sum".to_string());
    assert_eq!(table, None);
    assert_eq!(attr, "attribute_name".to_string());
    assert_pattern!(it.next(), None);
}

#[test]
fn test_parse_aggre_func() {
    test_aggre_func_with_table_name(AttributeExpr::parse_aggre_func);
    test_aggre_func_with_single_attr(AttributeExpr::parse_aggre_func);
    test_invalid_tokens(AttributeExpr::parse_aggre_func);
}

#[test]
fn test_attribute_parse() {
    test_single_attribute_name(AttributeExpr::parse);
    test_table_attribute(AttributeExpr::parse);
    test_aggre_func_with_table_name(AttributeExpr::parse);
    test_aggre_func_with_single_attr(AttributeExpr::parse);
    test_invalid_tokens(AttributeExpr::parse);
}

pub fn test_attr_list_helper(exp_list : AttributeList, correct_exp_list: Vec<(Option<&str>, Option<&str>, &str)>) {
    assert_eq!(exp_list.len(), correct_exp_list.len());
    let f = |s : &str| s.to_string();
    let correct_exp_list : Vec<_> = correct_exp_list.iter().map(
        |&(a, b, c)| (a.map(&f), b.map(&f), c.to_string())
    ).collect();
    let mut it = exp_list.iter();
    for exp in &correct_exp_list {
        match it.next().unwrap() {
            &AttributeExpr::TableAttr{ ref table, ref attr } =>
                assert_eq!((None, table.clone(), attr.clone()), *exp),
            &AttributeExpr::AggreFuncCall{ ref func, ref table, ref attr } =>
                assert_eq!((Some(func.clone()), table.clone(), attr.clone()), *exp),
        };
    }
}

#[test]
fn test_parse_list() {
    {
        let tokens = gen_token!("attribute_name");
        assert_eq!(tokens.len(), 1);
        let mut it = tokens.iter();
        let exp_list = AttributeExpr::parse_list(&mut it);
        assert_pattern!(exp_list, Ok(..));
        let exp_list = exp_list.unwrap();
        test_attr_list_helper(exp_list, vec![(None, None, "attribute_name")]);
        assert_pattern!(it.next(), None);
    }
    {
        let tokens = gen_token!("Huang, Guang.xing, sum(jiushi.doyoubi)");
        assert_eq!(tokens.len(), 12);
        let mut it = tokens.iter();
        let exp_list = AttributeExpr::parse_list(&mut it);
        assert_pattern!(exp_list, Ok(..));
        let exp_list = exp_list.unwrap();
        test_attr_list_helper(exp_list, vec![
            (None, None, "Huang"),
            (None, Some("Guang"), "xing"),
            (Some("sum"), Some("jiushi"), "doyoubi"),
        ]);
        assert_pattern!(it.next(), None);
    }
}
