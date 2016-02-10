use ::parser::common::exp_list_to_string;
use ::parser::select::{SelectExpr, Relation, GroupbyHaving, SelectStatement, RelationList};
use ::parser::attribute::AttributeExpr;
use ::parser::update::{AssignExpr, UpdateStatement};
use ::parser::insert::InsertStatement;
use ::parser::delete::DeleteStatement;
use ::parser::create_drop::{DropStatement, AttributeDeclaration, CreateStatement};
use super::super::utils::{test_by_display_str, test_by_list_to_str};

#[test]
fn test_parse_select_expr() {
    test_by_display_str(
        "select *", 2,
        SelectExpr::parse,
        "select *"
    );
    test_by_display_str(
        "select attribute_name", 2,
        SelectExpr::parse,
        "select attribute_name"
    );
    test_by_display_str(
        "select a1, a2, a3", 6,
        SelectExpr::parse,
        "select a1, a2, a3"
    );
}

#[test]
fn test_parse_relation() {
    test_by_list_to_str(
        "from table_name", 2,
        Relation::parse,
        "table_name"
    );
    test_by_list_to_str(
        "from tb1, tb2, tb3", 6,
        Relation::parse,
        "tb1, tb2, tb3"
    );
    test_by_list_to_str(
        "from (select * from tab)", 7,
        Relation::parse,
        "(select * from tab)"
    );
    test_by_list_to_str(
        "from tab, (select * from tab)", 9,
        Relation::parse,
        "tab, (select * from tab)"
    );
}

#[test]
fn test_parse_group_by_having() {
    test_by_display_str(
        "group by attribute", 3,
        GroupbyHaving::parse,
        "group by attribute"
    );
    test_by_display_str(
        "group by tab.attribute having dept.employee > 1", 11,
        GroupbyHaving::parse,
        "group by (tab.attribute) having ((dept.employee) > Integer(1))"
    );
}

#[test]
fn test_parse_select_statement() {
    test_by_display_str(
        "select sum(employee) from table_name where tab.money > 0\
        group by huang.guangxing having dept.number > 1 order by doyoubi",
        27,
        SelectStatement::parse,
        "select sum(employee) from table_name where ((tab.money) > Integer(0)) \
        group by (huang.guangxing) having ((dept.number) > Integer(1)) order by doyoubi"
    );
    test_by_display_str(
        "select tab.attr from huang group by doyoubi", 9,
        SelectStatement::parse,
        "select (tab.attr) from huang group by doyoubi"
    );
    test_by_display_str(
        "select attr from huang where doyoubi is not null", 8,
        SelectStatement::parse,
        "select attr from huang where (doyoubi is not Null(null))"
    );
    test_by_display_str(
        "select attr from huang order by doyoubi", 7,
        SelectStatement::parse,
        "select attr from huang order by doyoubi"
    );
}

#[test]
fn test_parse_assign() {
    test_by_list_to_str(
        "abc = 1", 3,
        AssignExpr::parse,
        "(abc = Integer(1))"
    );
}

#[test]
fn test_parse_assign_list() {
    test_by_list_to_str(
        "a = 1, b = 2", 7,
        AssignExpr::parse,
        "(a = Integer(1)), (b = Integer(2))"
    );
}

#[test]
fn test_udpate_statement_parse() {
    test_by_display_str(
        "update tab set a = 1", 6,
        UpdateStatement::parse,
        "update tab set (a = Integer(1))"
    );
    test_by_display_str(
        "update tab set a = 1, b = \"string\" where a > 1", 14,
        UpdateStatement::parse,
        "update tab set (a = Integer(1)), (b = String(string)) where (a > Integer(1))"
    );
}

#[test]
fn test_insert_statement_parse() {
    test_by_display_str(
        "insert tab values(1)", 6,
        InsertStatement::parse,
        "insert tab values(Integer(1))"
    );
    test_by_display_str(
        "insert tab values(1, null)", 8,
        InsertStatement::parse,
        "insert tab values(Integer(1), Null(null))"
    );
}

#[test]
fn test_delete_statement_parse() {
    test_by_display_str(
        "delete from tab", 3,
        DeleteStatement::parse,
        "delete from tab"
    );
    test_by_display_str(
        "delete from tab where a > 1", 7,
        DeleteStatement::parse,
        "delete from tab where (a > Integer(1))"
    );
}

#[test]
fn test_drop_statement_parse() {
    test_by_display_str(
        "drop table dept", 3,
        DropStatement::parse,
        "drop table dept"
    );
}

#[test]
fn test_attribute_declaration_parse() {
    test_by_display_str(
        "name char not null", 4,
        AttributeDeclaration::parse_decl,
        "(name String not null)"
    );
    test_by_display_str(
        "name char null", 3,
        AttributeDeclaration::parse_decl,
        "(name String null)"
    );
    test_by_display_str(
        "name char primary", 3,
        AttributeDeclaration::parse_decl,
        "(name String null primary)"
    );
}

#[test]
fn test_attr_decl_list() {
    test_by_list_to_str(
        "title char not null, content char", 7,
        AttributeDeclaration::parse_list,
        "(title String not null), (content String null)"
    );
}

#[test]
fn test_create_statement_parse() {
    test_by_display_str(
        "create dept (\
            id int primary,\
            name char not null\
        )", 12,
        CreateStatement::parse,
        "create dept ((id Integer null primary), (name String not null))"
    )
}
