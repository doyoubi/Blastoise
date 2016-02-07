use std::vec::Vec;
use std::option::Option::{Some, None};
use ::parser::common::AttributeExpr;
use ::parser::common::ConditionExpr;

struct SelectStatement {
    select_expr : SelectExpr,
    relation_list : Vec<Relation>,
    where_condition : Option<ConditionExpr>,
    groupby_having : Option<GroupbyHaving>,
    order_by_attr : Option<Attribute>,
}

enum SelectExpr {
    AllAttribute,
    AttrList(Vec<String>),
}

enum Relation {
    TableName(String),
    Select(SelectStatement),
}

struct GroupbyHaving {
    attr : String,
    having_condition : Option<ConditionExpr>,
}
