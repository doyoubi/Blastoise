use ::parser::common::exp_list_to_string;
use ::parser::select::{SelectExpr, Relation, GroupbyHaving, SelectStatement, RelationList};
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
    {
        let tokens = gen_token!("select a1, a2, a3");
        assert_eq!(tokens.len(), 6);
        let mut it = tokens.iter();
        let exp = SelectExpr::parse(&mut it);
        assert_pattern!(exp, Ok(..));
        let exp = exp.unwrap();
        assert_eq!(format!("{}", exp), "select a1, a2, a3");
        assert_pattern!(it.next(), None);
    }
}

#[test]
fn test_parse_relation() {
    {
        let tokens = gen_token!("from table_name");
        assert_eq!(tokens.len(), 2);
        let mut it = tokens.iter();
        let exp_list = Relation::parse(&mut it);
        let exp_list = extract!(exp_list, Ok(exp_list), exp_list);
        assert_eq!(exp_list_to_string(&exp_list), "table_name");
        assert_pattern!(it.next(), None);
    }
    {
        let tokens = gen_token!("from tb1, tb2, tb3");  // TODO: add sub select
        assert_eq!(tokens.len(), 6);
        let mut it = tokens.iter();
        let exp_list = Relation::parse(&mut it);
        let exp_list = extract!(exp_list, Ok(exp_list), exp_list);
        assert_eq!(exp_list_to_string(&exp_list), "tb1, tb2, tb3");
        assert_pattern!(it.next(), None);
    }
}

#[test]
fn test_parse_group_by_having() {
    {
        let tokens = gen_token!("group by attribute");
        assert_eq!(tokens.len(), 3);
        let mut it = tokens.iter();
        let exp = GroupbyHaving::parse(&mut it);
        let (attr, having_condition) = extract!(
            exp, Ok(GroupbyHaving{ attr, having_condition }), (attr, having_condition));
        assert_eq!(format!("{}", attr), "attribute");
        assert_pattern!(having_condition, None);
        assert_pattern!(it.next(), None);
    }
    {
        let tokens = gen_token!("group by tab.attribute having dept.employee > 1");
        assert_eq!(tokens.len(), 11);
        let mut it = tokens.iter();
        let exp = GroupbyHaving::parse(&mut it);
        let (attr, having_condition) = extract!(
            exp, Ok(GroupbyHaving{ attr, having_condition }), (attr, having_condition));
        assert_eq!(format!("{}", attr), "(tab.attribute)");
        assert_pattern!(having_condition, Some(..));
        assert_pattern!(it.next(), None);
    }
}

#[test]
fn test_parse_select_statement() {
    {
        let tokens = gen_token!(
            "select sum(employee) from table_name where tab.money > 0\
            group by huang.guangxing having dept.number > 1 order by doyoubi");
        assert_eq!(tokens.len(), 27);
        let mut it = tokens.iter();
        let select = SelectStatement::parse(&mut it);
        let select = extract!(select, Ok(select), select);
        assert_eq!(format!("{}", select),
            "select sum(employee) from table_name where ((tab.money) > Integer(0)) \
            group by (huang.guangxing) having ((dept.number) > Integer(1)) order by doyoubi");
    }
    {
        let tokens = gen_token!("select tab.attr from huang group by doyoubi");
        assert_eq!(tokens.len(), 9);
        let mut it = tokens.iter();
        let select = SelectStatement::parse(&mut it);
        let select = extract!(select, Ok(select), select);
        assert_eq!(format!("{}", select), "select (tab.attr) from huang group by doyoubi");
    }
    {
        let tokens = gen_token!("select attr from huang where doyoubi is not null");
        assert_eq!(tokens.len(), 8);
        let mut it = tokens.iter();
        let select = SelectStatement::parse(&mut it);
        let select = extract!(select, Ok(select), select);
        assert_eq!(format!("{}", select), "select attr from huang where (doyoubi is not Null(null))");
    }
    {
        let tokens = gen_token!("select attr from huang order by doyoubi");
        assert_eq!(tokens.len(), 7);
        let mut it = tokens.iter();
        let select = SelectStatement::parse(&mut it);
        let select = extract!(select, Ok(select), select);
        assert_eq!(format!("{}", select), "select attr from huang order by doyoubi");
    }
}
