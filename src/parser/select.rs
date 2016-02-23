use std::fmt;
use std::fmt::{Formatter, Display};
use std::vec::Vec;
use std::option::Option::{Some, None};
use super::lexer::{TokenIter, TokenType};
use super::attribute::{AttributeExpr, AttributeList};
use super::condition::ConditionExpr;
use super::compile_error::ErrorList;
use super::common::{
    get_next_token,
    consume_next_token_with_type,
    check_parse_to_end,
    seq_parse_helper,
    exp_list_to_string,
    concat_format,
    concat_error_list,
    parse_list_helper,
    align_iter,
};


#[derive(Debug)]
pub struct SelectStatement {
    pub select_expr : SelectExpr,
    pub relation_list : Vec<Relation>,
    pub where_condition : Option<ConditionExpr>,
    pub groupby_having : Option<GroupbyHaving>,
    pub order_by_attr : Option<AttributeExpr>,
}

impl Display for SelectStatement {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        let mut s = format!("{} from {}", self.select_expr, exp_list_to_string(&self.relation_list));
        s = concat_format(s, "where ", &self.where_condition);
        s = concat_format(s, "", &self.groupby_having);
        s = concat_format(s, "order by ", &self.order_by_attr);
        write!(f, "{}", s)
    }
}

impl SelectStatement {
    pub fn parse_as_sub_relation(it : &mut TokenIter) -> Result<SelectStatement, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::OpenBracket));
        let select_expr = try!(SelectExpr::parse(it));
        let relation_list = try!(Relation::parse(it));
        let (where_condition, es1) = seq_parse_helper(SelectStatement::parse_where, it);
        let (groupby_having, es2) = seq_parse_helper(GroupbyHaving::parse, it);
        let (order_by_attr, es3) = seq_parse_helper(SelectStatement::parse_order_by, it);
        match consume_next_token_with_type(it, TokenType::CloseBracket) {
            Err(errs) => Err(concat_error_list(vec![errs, es1, es2, es3])),
            Ok(..) => Ok(SelectStatement {
                    select_expr : select_expr,
                    relation_list : relation_list,
                    where_condition : where_condition,
                    groupby_having : groupby_having,
                    order_by_attr : order_by_attr,
                })
        }
    }
    pub fn parse(it : &mut TokenIter) -> Result<SelectStatement, ErrorList> {
        let select_expr = try!(SelectExpr::parse(it));
        let relation_list = try!(Relation::parse(it));
        let (where_condition, es1) = seq_parse_helper(SelectStatement::parse_where, it);
        let (groupby_having, es2) = seq_parse_helper(GroupbyHaving::parse, it);
        let (order_by_attr, es3) = seq_parse_helper(SelectStatement::parse_order_by, it);
        match check_parse_to_end(it) {
            Some(err) => Err(concat_error_list(vec![vec![err], es1, es2, es3])),
            None => Ok(SelectStatement {
                select_expr : select_expr,
                relation_list : relation_list,
                where_condition : where_condition,
                groupby_having : groupby_having,
                order_by_attr : order_by_attr,
            }),
        }
    }
    pub fn parse_where(it : &mut TokenIter) -> Result<ConditionExpr, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Where));
        ConditionExpr::parse(it)
    }
    pub fn parse_order_by(it : &mut TokenIter) -> Result<AttributeExpr, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Order));
        try!(consume_next_token_with_type(it, TokenType::By));
        AttributeExpr::parse(it)
    }
}

#[derive(Debug)]
pub enum SelectExpr {
    AllAttribute,
    AttrList(AttributeList),
}

impl SelectExpr {
    pub fn parse(it : &mut TokenIter) -> Result<SelectExpr, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Select));
        let token = try!(get_next_token(it));
        match token.token_type {
            TokenType::Star => {
                it.next();
                Ok(SelectExpr::AllAttribute)
            }
            _ => Ok(SelectExpr::AttrList(try!(AttributeExpr::parse_list(it))))
        }
    }
}

impl Display for SelectExpr {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        match self {
            &SelectExpr::AllAttribute => write!(f, "select *"),
            &SelectExpr::AttrList(ref attr_list) => write!(f, "select {}", exp_list_to_string(attr_list)),
        }
    }
}


pub type RelationList = Vec<Relation>;

#[derive(Debug)]
pub enum Relation {
    TableName(String),
    Select(SelectStatement),
}

impl Display for Relation {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        match self {
            &Relation::TableName(ref name) => write!(f, "{}", name),
            &Relation::Select(ref select) => write!(f, "({})", select),
        }
    }
}

impl Relation {
    pub fn parse(it : &mut TokenIter) -> Result<RelationList, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::From));
        Relation::parse_list(it)
    }
    pub fn parse_list(it : &mut TokenIter) -> Result<RelationList, ErrorList> {
        parse_list_helper(Relation::parse_relation, it)
    }
    pub fn parse_relation(it : &mut TokenIter) -> Result<Relation, ErrorList> {
        let token = try!(get_next_token(it));
        match token.token_type {
            TokenType::OpenBracket =>
                Ok(Relation::Select(try!(SelectStatement::parse_as_sub_relation(it)))),
            _ => {
                let token = try!(consume_next_token_with_type(it, TokenType::Identifier));
                Ok(Relation::TableName(token.value.clone()))
            }
        }
    }
}

#[derive(Debug)]
pub struct GroupbyHaving {
    pub attr : AttributeExpr,
    pub having_condition : Option<ConditionExpr>,
}

impl Display for GroupbyHaving {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        match &self.having_condition {
            &Some(ref cond_expr) => write!(f, "group by {} having {}", self.attr, cond_expr),
            &None => write!(f, "group by {}", self.attr),
        }
    }
}

impl GroupbyHaving {
    pub fn parse(it : &mut TokenIter) -> Result<GroupbyHaving, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Group));
        try!(consume_next_token_with_type(it, TokenType::By));
        let attr = try!(AttributeExpr::parse(it));
        let mut tmp = it.clone();
        match GroupbyHaving::parse_having(&mut tmp) {
            Ok(cond) => {
                align_iter(it, &mut tmp);
                Ok(GroupbyHaving{ attr : attr, having_condition : Some(cond) })
            }
            Err(..) => Ok(GroupbyHaving{ attr : attr, having_condition : None }),
        }
    }

    pub fn parse_having(it : &mut TokenIter) -> Result<ConditionExpr, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Having));
        ConditionExpr::parse(it)
    }
}
