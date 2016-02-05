use std::option::Option::{Some, None};
use std::result::Result::Ok;
use ::parser::condition::ArithExpr;
use ::parser::common::{
    parse_table_attr,
};

#[test]
fn test_parse_table_attr() {
    {
        // test attribute without table name
        let tokens = gen_token!("attribute_name");
        assert_eq!(tokens.len(), 1);
        let mut it = tokens.iter();
        let arith_exp = parse_table_attr(&mut it);
        assert_pattern!(arith_exp, Ok(_));
        let arith_exp = arith_exp.unwrap();
        let (table, attr) = extract!(arith_exp, ArithExpr::TableAttr{ table, attr }, (table, attr));
        assert!(!table.is_some());
        assert_eq!(attr, "attribute_name".to_string());
        assert_pattern!(it.next(), None);
    }
    {
        // test attribute with table name
        let tokens = gen_token!("table_name.attribute_name");
        assert_eq!(tokens.len(), 3);
        let mut it = tokens.iter();
        let arith_exp = parse_table_attr(&mut it);
        assert_pattern!(arith_exp, Ok(_));
        let arith_exp = arith_exp.unwrap();
        let (table, attr) = extract!(arith_exp, ArithExpr::TableAttr{ table, attr }, (table, attr));
        assert_eq!(table, Some("table_name".to_string()));
        assert_eq!(attr, "attribute_name".to_string());
        assert_pattern!(it.next(), None);
    }
}
