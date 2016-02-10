use std::fmt;
use std::fmt::{Formatter, Display};
use std::option::Option::{Some, None};
use super::lexer::{TokenIter, TokenType};
use super::compile_error::ErrorList;
use super::common::{
    ValueType,
    consume_next_token_with_type,
    consume_next_token_with_type_list,
    check_parse_to_end,
    exp_list_to_string,
    parse_list_helper,
    gen_data_type,
    seq_parse_helper,
};


#[derive(Debug)]
pub struct CreateStatement {
    pub table : String,
    pub decl_list : AttrDeclList,
}

impl Display for CreateStatement {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        write!(f, "create {} ({})", self.table, exp_list_to_string(&self.decl_list))
    }
}

impl CreateStatement {
    pub fn parse(it : &mut TokenIter) -> Result<CreateStatement, ErrorList> {
        try!(consume_next_token_with_type(it, TokenType::Create));
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

pub type AttrDeclList = Vec<AttributeDeclaration>;

#[derive(Debug)]
pub struct AttributeDeclaration {
    name : String,
    value_type : ValueType,
    nullable : bool,
    primary : bool,
}

impl Display for AttributeDeclaration {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        let null = if self.nullable {" null"} else {" not null"};
        let primary = if self.primary {" primary"} else {""};
        write!(f, "({} {:?}{}{})", self.name, self.value_type, null, primary)
    }
}

impl AttributeDeclaration {
    pub fn parse_list(it : &mut TokenIter) -> Result<AttrDeclList, ErrorList> {
        parse_list_helper(AttributeDeclaration::parse_decl, it)
    }
    pub fn parse_decl(it : &mut TokenIter) -> Result<AttributeDeclaration, ErrorList> {
        let table_token = try!(consume_next_token_with_type(it, TokenType::Identifier));
        let data_type_tokens = vec![TokenType::Int, TokenType::Float, TokenType::Char];
        let type_token = try!(consume_next_token_with_type_list(it, &data_type_tokens));
        let nullable = !is_match!(seq_parse_helper(
            AttributeDeclaration::parse_null_specifier, it), (Some(false), _));
        let primary = is_match!(seq_parse_helper(
            AttributeDeclaration::parse_primary, it), (Some(true), _));
        Ok(AttributeDeclaration{
            name : table_token.value.clone(),
            value_type : gen_data_type(type_token.token_type),
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
