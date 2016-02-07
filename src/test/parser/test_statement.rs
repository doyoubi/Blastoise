use ::parser::select::{SelectExpr, Relation, GroupbyHaving};
use ::parser::attribute::AttributeExpr;

#[test]
fn test_parse_select_expr() {
    {
        let tokens = gen_token!("select *");
        assert_eq!(tokens.len(), 2);
        let mut it = tokens.iter();
        let exp = SelectExpr::parse(&mut it);
        assert_pattern!(exp, Ok(..));
        let exp = exp.unwrap();
        assert_pattern!(exp, SelectExpr::AllAttribute);
    }
    {
        let tokens = gen_token!("select attribute_name");
        assert_eq!(tokens.len(), 2);
        let mut it = tokens.iter();
        let exp = SelectExpr::parse(&mut it);
        assert_pattern!(exp, Ok(..));
        let exp = exp.unwrap();
        let attr_list = extract!(exp, SelectExpr::AttrList(attr_list), attr_list);
        assert_eq!(attr_list.len(), 1);
        let attr = &attr_list[0];
        let (_, attr) = extract!(attr,
            &AttributeExpr::TableAttr{ ref table, ref attr }, (table.clone(), attr.clone()));
        assert_eq!(attr, "attribute_name");
        assert_pattern!(it.next(), None);
    }
}

#[test]
fn test_parse_relation() {
    let tokens = gen_token!("from table_name");
    assert_eq!(tokens.len(), 2);
    let mut it = tokens.iter();
    let exp = Relation::parse(&mut it);
    let name = extract!(exp, Ok(Relation::TableName(name)), name);
    assert_eq!(name, "table_name");
}

#[test]
fn test_parse_group_by_having() {
    {
        let tokens = gen_token!("having attribute");
        assert_eq!(tokens.len(), 2);
        let mut it = tokens.iter();
        let exp = GroupbyHaving::parse(&mut it);
        let (attr, having_condition) = extract!(
            exp, Ok(GroupbyHaving{ attr, having_condition }), (attr, having_condition));
        assert_eq!(attr, "attribute");
        assert_pattern!(having_condition, None);
    }
}
