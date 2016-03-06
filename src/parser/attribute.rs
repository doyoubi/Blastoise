use std::fmt;
use std::fmt::{Formatter, Display};
use std::option::Option::{Some, None};
use std::result::Result;
use std::result::Result::{Ok, Err};
use std::vec::Vec;
use super::lexer::{TokenIter, TokenType};
use super::compile_error::ErrorList;
use super::common::{
    consume_next_token_with_type,
    align_iter,
    parse_list_helper,
};


pub type ParseAttrResult = Result<AttributeExpr, ErrorList>;
pub type AttributeList = Vec<AttributeExpr>;

#[derive(Debug)]
pub enum AttributeExpr {
    TableAttr { table : Option<String>, attr : String },
    AggreFuncCall {
        func : String,
        table : Option<String>,
        attr : String,
    },
}

impl Display for AttributeExpr {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        match self {
            &AttributeExpr::TableAttr{ref table, ref attr} => {
                match table {
                    &Some(ref table) => write!(f, "({}.{})", table, attr),
                    &None => write!(f, "{}", attr),
                }
            }
            &AttributeExpr::AggreFuncCall{ref func, ref table, ref attr} => {
                match table {
                    &Some(ref table) => write!(f, "{}({}.{})", func, table, attr),
                    &None => write!(f, "{}({})", func, attr),
                }
            }
        }
    }
}

impl AttributeExpr {
    pub fn parse_list(it : &mut TokenIter) -> Result<AttributeList, ErrorList> {
        parse_list_helper(AttributeExpr::parse, it)
    }

    pub fn parse(it : &mut TokenIter) -> ParseAttrResult {
        or_parse_combine!(it,
            AttributeExpr::parse_aggre_func,
            AttributeExpr::parse_table_attr
        )
    }

    pub fn parse_table_attr(it : &mut TokenIter) -> ParseAttrResult  {
        let token = try!(consume_next_token_with_type(it, TokenType::Identifier));
        let mut look_ahead = it.clone();
        let next_token_type = look_ahead.next().map(|tk| tk.token_type);
        match next_token_type {
            Some(TokenType::GetMember) => {
                let third_token = try!(consume_next_token_with_type(&mut look_ahead, TokenType::Identifier));
                align_iter(it, &mut look_ahead);
                Ok(AttributeExpr::TableAttr{ table : Some(token.value.clone()), attr : third_token.value.clone() })
            }
            _ => Ok(AttributeExpr::TableAttr{ table : None, attr : token.value.clone() })
        }
    }

    pub fn parse_aggre_func(it : &mut TokenIter) -> ParseAttrResult {
        let func_token = try!(consume_next_token_with_type(it, TokenType::Identifier));
        try!(consume_next_token_with_type(it, TokenType::OpenBracket));
        let table_attr = try!(AttributeExpr::parse_table_attr(it));
        let (table_name, attr_name) = extract!(table_attr, AttributeExpr::TableAttr{ table, attr }, (table, attr));
        try!(consume_next_token_with_type(it, TokenType::CloseBracket));
        Ok(AttributeExpr::AggreFuncCall{
            func : func_token.value.clone(),
            table : table_name,
            attr : attr_name,
        })
    }
    
    pub fn get_attr(&mut self) -> (&mut Option<String>, &mut String) {
        match self {
            &mut AttributeExpr::TableAttr{ref mut table, ref mut attr} => (table, attr),
            &mut AttributeExpr::AggreFuncCall{ref mut table, ref mut attr, ..} => (table, attr),
        }
    }
}
