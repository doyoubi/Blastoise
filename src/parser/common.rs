use std::vec::Vec;
use std::rc::Rc;
use std::fmt::Display;
use std::result::Result::{Ok, Err};
use super::lexer::{Token, TokenRef, TokenType, TokenIter};
use super::condition::ArithExpr;
use super::compile_error::{CompileError, CompileErrorType, ErrorList};
use ::store::table::{TableRef, AttrRef};


#[derive(Copy, Clone)]
pub enum ValueType {
    Integer,
    Float,
    String,
    Null,
}

pub struct Parser {
    tables : Vec<TableRef>,
}

pub type ParseArithResult = std::Result<ArithExpr, ErrorList>;

impl Parser {
    fn get_table_attr(&self, table : String, attribute : String)
            -> Option<(TableRef, AttrRef)> {
        for tab in self.tables {
            if tab.name != table {
                continue;
            }
            for attr in tab.attrs {
                if attr.name == attribute {
                    return Some((tab.clone(), attr.clone()));
                }
            }
        }
        None
    }
}

// pub fn check_reach_the_end(it : &TokenIter) -> Option<ErrorList> {
//     if it.peekable().peek().is_some() {
//         return None;
//     }
//     gen_reach_the_end_err(it, );
// }

fn gen_end_token(it : &TokenIter) -> TokenRef {
    let it = it.clone();
    match it.next_back() {
        Some(token_ref) => token_ref,
        // dummy token
        None => Rc::new(Token{
            column : 0,
            value : "".to_string(),
            token_type : TokenType::UnKnown
        }),
    }
}

fn gen_reach_the_end_err_with_type(it : &TokenIter, expected_token_type : TokenType) -> ErrorList {
    let token = gen_end_token(it);
    let err_msg = format!("expect {} but no more token found", expected_token_type);
    let err = Rc::new(CompileError{
        error_type : CompileErrorType::ParserNoMoreToken,
        token : token,
        error_msg : err_msg,
    });
    vec![err]
}

fn gen_reach_the_end_err(it : &TokenIter) -> ErrorList {
    let token = gen_end_token(it);
    let error_msg = format!("expect token but no more token found");
    let err = Rc::new(CompileError{
        error_type : CompileErrorType::ParserNoMoreToken,
        token : token,
        error_msg : err_msg,
    });
    vec![err]
}

pub fn check_single_token_type(it : &TokenIter, token_type : TokenType) -> Option<ErrorList> {
    // if let Some(errs) = check_reach_the_end(it) {
    //     return Some(errs);
    // }
    // if it.peekable().peek().is_some() {
    //     return gen_reach_the_end_err(it, token_type);
    // }
    // let token = it.peekable().peek();
    let token = match it.peekable().peek() {
        Some(token) => token,
        None => return Some(gen_reach_the_end_err_with_type(it, token_type)),
    };
    if let TokenType = token.token_type {
        return None;
    }
    let err_msg = format!("expect token type: {:?}, but got {:?}",
        token_type, token.token_type);
    let err = Rc::new(CompileError{
        error_type : CompileErrorType::ParserUnExpectedTokenType,
        token : token,
        error_msg : err_msg,
    });
    Some(vec![err])
}

pub fn parse_single_token_type(it : &mut TokenIter, token_type : TokenType) -> Option<ErrorList> {
    match check_single_token_type(it, token_type) {
        None => {
            it.next();
            None
        }
        some_errs => some_errs,
    }
}

pub fn consume_next_token_with_type(it : &mut TokenIter, token_type : TokenType)
        -> Result<TokenRef, ErrorList> {
    match check_single_token_type(it, token_type) {
        None => Ok(it.next()),
        Some(errs) => Err(errs),
    }
}

pub fn consume_next_token(it : &mut TokenIter)
        -> Result<TokenRef, ErrorList> {
    match it.next() {
        Some(token) => Ok(token),
        None => Err(gen_reach_the_end_err(it, token_type)),
    }
}

pub fn parse_table_attr(it : &mut TokenIter, parser : &Parser) -> ParseArithResult {
    // if let Some(errs) = check_single_token_type(it, TokenType::Identifier) {
        // return Some(errs);
    // }
    // let token = it.peekable().peek();
    // let token = match consume_next_token(it, TokenType::Identifier) {
    //     Ok(token) => token,
    //     Err(errs) => Err(errs),
    // }
    let token = try!(consume_next_token_with_type(it, TokenType::Identifier));
    it.next_back();
    let look_ahead = it.clone().peekable();
    let next_token_type = look_ahead.next().next().map(|tk| tk.token_type);
    let err_msg : String;
    match next_token_type {
        Some(TokenType::GetMember) => {
            if let Some(errs) = check_single_token_type(&look_ahead, TokenType::Identifier) {
                return Err(errs);
            }
            if let Some((table, attr)) = parser.get_table_attr(
                    token.value, look_ahead.peek().value) {
                it.next().next().next();
                return Ok(ArithExpr::TableAttr{ table : table, attr : attr });
            }
            err_msg = format!("{}.{} does not exist", token.value, look_ahead.peek().value);
        }
        _ => {
            let err = match parser.tables.len() {
                len if len == 0 => Some((CompileErrorType::ParserNoTable,
                    "no table specified in `from` clause")),
                len if len > 1 => Some((CompileErrorType::ParserLackOfSpecifyingTable,
                    "should specify table name when `from` clause have multiple tables")),
                _ => None,
            };
            if let Some((err_type, err_msg)) = err {
                let e = CompileError{
                    error_type : err_type,
                    token : token,
                    error_msg : err_msg,
                };
                return Err(vec![Rc::new(e)]);
            }
            if let Some((table, attr)) = parser.get_table_attr(None, token.value) {
                it.next();
                return Ok(ArithExpr::TableAttr{ table : table, attr : attr });
            }
            err_msg = format!("attribute {} does not exist", token.value);
        }
    }
    let err = CompileError{
        error_type : CompileErrorType::ParserTableAttrNotExist,
        token : token,
        error_msg : err_msg,
    };
    Err(vec![Rc::new(err)])
}
