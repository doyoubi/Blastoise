use std::option::Option::{Some, None};
use std::result::Result::Ok;
use ::parser::attribute::AttributeExpr;
use ::parser::compile_error::CompileErrorType;


#[test]
fn test_parse_table_attr() {
    {
        // test attribute without table name
        let tokens = gen_token!("attribute_name");
        assert_eq!(tokens.len(), 1);
        let mut it = tokens.iter();
        let attr_exp = AttributeExpr::parse_table_attr(&mut it);
        assert_pattern!(attr_exp, Ok(_));
        let attr_exp = attr_exp.unwrap();
        let (table, attr) = extract!(attr_exp, AttributeExpr::TableAttr{ table, attr }, (table, attr));
        assert!(!table.is_some());
        assert_eq!(attr, "attribute_name".to_string());
        assert_pattern!(it.next(), None);
    }
    {
        // test attribute with table name
        let tokens = gen_token!("table_name.attribute_name");
        assert_eq!(tokens.len(), 3);
        let mut it = tokens.iter();
        let attr_exp = AttributeExpr::parse_table_attr(&mut it);
        assert_pattern!(attr_exp, Ok(_));
        let attr_exp = attr_exp.unwrap();
        let (table, attr) = extract!(attr_exp, AttributeExpr::TableAttr{ table, attr }, (table, attr));
        assert_eq!(table, Some("table_name".to_string()));
        assert_eq!(attr, "attribute_name".to_string());
        assert_pattern!(it.next(), None);
    }
    {
        // test fail
        let tokens = gen_token!("1");
        assert_eq!(tokens.len(), 1);
        let mut it = tokens.iter();
        assert_eq!(it.len(), 1);
        let attr_exp = AttributeExpr::parse_table_attr(&mut it);
        assert_pattern!(attr_exp, Err(_));
        let ref err = attr_exp.unwrap_err()[0];
        assert_eq!(err.error_type, CompileErrorType::ParserUnExpectedTokenType);
        assert_eq!(it.len(), 1);
    }
}
