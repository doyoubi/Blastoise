use std::rc::Rc;
use std::result::Result::{Ok, Err};
use super::lexer::{Token, TokenRef, TokenType, TokenIter};
use super::condition::ArithExpr;
use super::compile_error::{CompileError, CompileErrorType, ErrorList};


#[allow(dead_code)]  // lint bug
#[derive(Copy, Clone, Debug)]
pub enum ValueType {
    Integer,
    Float,
    String,
    Null,
}

pub type ParseArithResult = Result<ArithExpr, ErrorList>;

fn gen_end_token(it : &TokenIter) -> TokenRef {
    let mut it = it.clone();
    match it.next_back() {
        Some(token_ref) => token_ref.clone(),
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
    let err_msg = format!("expect {:?} but no more token found", expected_token_type);
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
        error_msg : error_msg,
    });
    vec![err]
}

pub fn get_single_token_by_type(it : &TokenIter, token_type : TokenType)
        -> Result<TokenRef, ErrorList> {
    let token : TokenRef = match it.clone().peekable().peek() {
        Some(token) => (*token).clone(),
        None => return Err(gen_reach_the_end_err_with_type(it, token_type)),
    };
    if token_type == token.token_type {
        return Ok(token);
    }
    let err_msg = format!("expect token type: {:?}, but got {:?}",
        token_type, token.token_type);
    let err = Rc::new(CompileError{
        error_type : CompileErrorType::ParserUnExpectedTokenType,
        token : token.clone(),
        error_msg : err_msg,
    });
    Err(vec![err])
}

pub fn consume_next_token_with_type(it : &mut TokenIter, token_type : TokenType)
        -> Result<TokenRef, ErrorList> {
    match get_single_token_by_type(it, token_type) {
        Ok(token) => {
            it.next();
            Ok(token)
        }
        errs => errs,
    }
}

pub fn consume_next_token(it : &mut TokenIter)
        -> Result<TokenRef, ErrorList> {
    match it.next() {
        Some(token) => Ok(token.clone()),
        None => Err(gen_reach_the_end_err(it)),
    }
}

pub fn parse_table_attr(it : &mut TokenIter) -> ParseArithResult {
    let token = try!(consume_next_token_with_type(it, TokenType::Identifier));
    it.next_back();
    let mut look_ahead = it.clone();
    let next_token_type = look_ahead.nth(2).map(|tk| tk.token_type);
    match next_token_type {
        Some(TokenType::GetMember) => {
            let third_token = try!(get_single_token_by_type(&look_ahead, TokenType::Identifier));
            it.nth(3);
            Ok(ArithExpr::TableAttr{ table : Some(token.value.clone()), attr : third_token.value.clone() })
        }
        _ => Ok(ArithExpr::TableAttr{ table : None, attr : token.value.clone() })
    }
}
