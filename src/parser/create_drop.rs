use std::fmt;
use std::fmt::{Formatter, Display};
use std::option::Option::{Some, None};
use super::lexer::{TokenIter, TokenType};
use super::compile_error::ErrorList;
use super::common::{
    consume_next_token_with_type,
    consume_next_token_with_type_list,
    check_parse_to_end,
    exp_list_to_string,
    parse_list_helper,
    seq_parse_helper,
};


#[derive(Debug)]
pub struct CreateStatement {
    pub table : String,
    pub decl_list : AttrDeclList,
}

impl Display for CreateStatement {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        write!(f, "create table {} ({})", self.table, exp_list_to_string(&self.decl_list))
    }
}

impl CreateStatement {
    pub fn parse(it : &mut TokenIter) -> Result<CreateStatement, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Create));
        try!(consume_next_token_with_type(it, TokenType::Table));
        let table_token = try!(consume_next_token_with_type(it, TokenType::Identifier));
        try!(consume_next_token_with_type(it, TokenType::OpenBracket));
        let decl_list = try!(AttributeDeclaration::parse_list(it));
        try!(consume_next_token_with_type(it, TokenType::CloseBracket));
        match check_parse_to_end(it) {
            Some(err) => Err(vec![err]),
            None => Ok(CreateStatement {
                table : table_token.value.clone(),
                decl_list : decl_list,
            }),
        }
    }
}

#[derive(Debug)]
pub enum AttrType {
    Int,
    Float,
    Char{ len : String },
}

impl Display for AttrType {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        match self {
            &AttrType::Int => write!(f, "Int"),
            &AttrType::Float => write!(f, "Float"),
            &AttrType::Char{ ref len } => write!(f, "Char({})", len),
        }
    }
}

impl AttrType {
    pub fn parse(it : &mut TokenIter) -> Result<AttrType, ErrorList> {
        let data_type_tokens = vec![TokenType::Int, TokenType::Float, TokenType::Char];
        let token = try!(consume_next_token_with_type_list(it, &data_type_tokens));
        match token.token_type {
            TokenType::Int => Ok(AttrType::Int),
            TokenType::Float => Ok(AttrType::Float),
            TokenType::Char => {
                try!(consume_next_token_with_type(it, TokenType::OpenBracket));
                let len_token = try!(consume_next_token_with_type(it, TokenType::IntegerLiteral));
                try!(consume_next_token_with_type(it, TokenType::CloseBracket));
                Ok(AttrType::Char{ len : len_token.value.clone() })
            }
            other => panic!("unexpected token: {:?}", other),
        }
    }
}

pub type AttrDeclList = Vec<AttributeDeclaration>;

#[derive(Debug)]
pub struct AttributeDeclaration {
    pub name : String,
    pub attr_type : AttrType,
    pub nullable : bool,
    pub primary : bool,
}

impl Display for AttributeDeclaration {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        let null = if self.nullable {" null"} else {" not null"};
        let primary = if self.primary {" primary"} else {""};
        write!(f, "({} {}{}{})", self.name, self.attr_type, null, primary)
    }
}

impl AttributeDeclaration {
    pub fn parse_list(it : &mut TokenIter) -> Result<AttrDeclList, ErrorList> {
        parse_list_helper(AttributeDeclaration::parse_decl, it)
    }
    pub fn parse_decl(it : &mut TokenIter) -> Result<AttributeDeclaration, ErrorList> {
        let table_token = try!(consume_next_token_with_type(it, TokenType::Identifier));
        let attr_type = try!(AttrType::parse(it));
        let nullable = !is_match!(seq_parse_helper(
            AttributeDeclaration::parse_null_specifier, it), (Some(false), _));
        let primary = is_match!(seq_parse_helper(
            AttributeDeclaration::parse_primary, it), (Some(true), _));
        Ok(AttributeDeclaration{
            name : table_token.value.clone(),
            attr_type : attr_type,
            nullable : nullable,
            primary : primary,
        })
    }
    fn parse_primary(it : &mut TokenIter) -> Result<bool, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Primary));
        Ok(true)
    }
    fn parse_null_specifier(it : &mut TokenIter) -> Result<bool, ErrorList> {
        or_parse_combine!(it,
            AttributeDeclaration::parse_null,
            AttributeDeclaration::parse_not_null
        )
    }
    fn parse_null(it : &mut TokenIter) -> Result<bool, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Null));
        Ok(true)
    }
    fn parse_not_null(it : &mut TokenIter) -> Result<bool, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Not));
        try!(consume_next_token_with_type(it, TokenType::Null));
        Ok(false)
    }
}

#[derive(Debug)]
pub struct DropStatement {
    pub table : String,
}

impl Display for DropStatement {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        write!(f, "drop table {}", self.table)
    }
}

impl DropStatement {
    pub fn parse(it : &mut TokenIter) -> Result<DropStatement, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Drop));
        try!(consume_next_token_with_type(it, TokenType::Table));
        let table_token = try!(consume_next_token_with_type(it, TokenType::Identifier));
        match check_parse_to_end(it) {
            Some(err) => Err(vec![err]),
            None => Ok(DropStatement{
                table : table_token.value.clone(),
            })
        }
    }
}
