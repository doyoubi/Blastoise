use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::result::Result::{Ok, Err};
use std::iter::ExactSizeIterator;
use super::lexer::{Token, TokenRef, TokenType, TokenIter};
use super::compile_error::{CompileError, CompileErrorType, ErrorRef, ErrorList};
use super::select::SelectStatement;
use super::update::UpdateStatement;
use super::insert::InsertStatement;
use super::delete::DeleteStatement;
use super::create_drop::{CreateStatement, DropStatement};


#[allow(dead_code)]  // lint bug
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum ValueType {
    Integer,
    Float,
    String,
    Null,
}

pub type ValueList = Vec<ValueExpr>;

#[derive(Debug)]
pub struct ValueExpr {
    pub value : String,
    pub value_type : ValueType,
}

impl Display for ValueExpr {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        write!(f, "{:?}({})", self.value_type, self.value)
    }
}

impl ValueExpr {
    pub fn parse(it : &mut TokenIter) -> Result<ValueExpr, ErrorList> {
        let literals = vec![
            TokenType::IntegerLiteral,
            TokenType::FloatLiteral,
            TokenType::StringLiteral,
            TokenType::Null,
        ];
        let token = try!(consume_next_token_with_type_list(it, &literals));
        Ok(ValueExpr{
            value : token.value.clone(),
            value_type : token_type_to_value_type(token.token_type),
        })
    }
}

fn token_type_to_value_type(t : TokenType) -> ValueType {
    match t {
        TokenType::IntegerLiteral => ValueType::Integer,
        TokenType::FloatLiteral => ValueType::Float,
        TokenType::StringLiteral => ValueType::String,
        TokenType::Null => ValueType::Null,
        _ => panic!("unexpected TokenType: {:?}", t),
    }
}

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

pub fn consume_next_token_with_type(it : &mut TokenIter, token_type : TokenType)
        -> Result<TokenRef, ErrorList> {
    let token : TokenRef = match it.next() {
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

pub fn consume_next_token_with_type_list(it : &mut TokenIter, type_list : &Vec<TokenType>)
        -> Result<TokenRef, ErrorList> {
    let token : TokenRef = match it.next() {
        Some(token) => (*token).clone(),
        None => return Err(gen_reach_the_end_err(it)),
    };
    if type_list.contains(&token.token_type) {
        return Ok(token);
    }
    let err_msg = format!("expect token type: {:?}, but got {:?}",
        type_list, token.token_type);
    let err = Rc::new(CompileError{
        error_type : CompileErrorType::ParserUnExpectedTokenType,
        token : token.clone(),
        error_msg : err_msg,
    });
    Err(vec![err])
}

pub fn get_next_token(it : &TokenIter) -> Result<TokenRef, ErrorList> {
    match it.clone().peekable().peek() {
        Some(token) => Ok((*token).clone()),
        None => Err(gen_reach_the_end_err(it)),
    }
}

pub fn consume_next_token(it : &mut TokenIter)
        -> Result<TokenRef, ErrorList> {
    match it.next() {
        Some(token) => Ok(token.clone()),
        None => Err(gen_reach_the_end_err(it)),
    }
}

pub fn check_parse_to_end(it : &TokenIter) -> Option<ErrorRef> {
    match it.clone().peekable().peek() {
        None => None,
        Some(token) => Some(Rc::new(CompileError{
            error_type : CompileErrorType::ParserCanNotParseLeftToken,
            token : (*token).clone(),
            error_msg : format!("Can not parse the left tokens : {}", token.value),
        })),
    }
}

pub fn align_iter(it1 : &mut TokenIter, it2 : &mut TokenIter) {
    let (l1, l2) = (it1.len(), it2.len());
    if l1 < l2 {
        return align_iter(it2, it1);
    }
    if l1 - l2 > 0 {
        it1.nth(l1 - l2 - 1);
    }
}

macro_rules! try_parse_helper {
    ($result:expr, $iter:expr, $tmp:expr) => ({
        use std::result::Result::{Ok, Err};
        use std::convert::From;
        match $result {
            Ok(val) => {
                ::parser::common::align_iter($iter, &mut $tmp);
                val
            },
            Err(err) => {
                return Err(From::from(err))
            },
        }
    });
}

// when success, iter will be changed
// while not, iter stay still
macro_rules! try_parse {
    ($parse_func:ident, $iter:expr) => ({
        let mut tmp = $iter.clone();
        try_parse_helper!($parse_func(&mut tmp), $iter, tmp)
    });
    ($parse_func:ident, $iter:expr, $( $add_args:expr ),*) => ({
        let mut tmp = $iter.clone();
        try_parse_helper!($parse_func(&mut tmp, $($add_args),* ), $iter, tmp)
    });
    ($type_name:ident :: $parse_func:ident, $iter:expr) => ({
        let mut tmp = $iter.clone();
        try_parse_helper!($type_name::$parse_func(&mut tmp), $iter, tmp)
    });
    ($type_name:ident :: $parse_func:ident, $iter:expr, $( $add_args:expr ),*) => ({
        let mut tmp = $iter.clone();
        try_parse_helper!($type_name::$parse_func(&mut tmp, $($add_args),* ), $iter, tmp)
    });
}

macro_rules! or_parse_helper {
    ($result:expr, $iter:expr, $tmp:expr) => ({
        use std::result::Result::{Ok, Err};
        match $result {
            Ok(val) => {
                ::parser::common::align_iter($iter, &mut $tmp);
                return Ok(val)
            },
            Err(err) => {
                err
            },
        }
    });
}

// when success, iter will be changed and return the parsed Expr
// while not, iter stay still
macro_rules! or_parse {
    ($parse_func:ident, $iter:expr) => ({
        let mut tmp = $iter.clone();
        or_parse_helper!($parse_func(&mut tmp), $iter, tmp)
    });
    ($parse_func:ident, $iter:expr, $( $add_args:expr ),*) => ({
        let mut tmp = $iter.clone();
        or_parse_helper!($parse_func(&mut tmp, $($add_args),* ), $iter, tmp)
    });
    ($type_name:ident :: $parse_func:ident, $iter:expr) => ({
        let mut tmp = $iter.clone();
        or_parse_helper!($type_name::$parse_func(&mut tmp), $iter, tmp)
    });
    ($type_name:ident :: $parse_func:ident, $iter:expr, $( $add_args:expr ),*) => ({
        let mut tmp = $iter.clone();
        or_parse_helper!($type_name::$parse_func(&mut tmp, $($add_args),* ), $iter, tmp)
    });
}

macro_rules! or_parse_combine {
    ($iter:expr, $( $type_name:ident :: $parse_func:ident ),+ ) => ({
        use std::vec::Vec;
        let mut errors = Vec::new();
        $(
            let errs = or_parse!($type_name::$parse_func, $iter);
            ::parser::common::extend_from_other(&mut errors, &errs);
        )+
        Err(errors)
    });
}

// should be replaced when upgrade rust to 1.6
pub fn extend_from_other(errors : &mut Vec<ErrorRef>, other : &Vec<ErrorRef>) {
    errors.extend(other.iter().cloned());
}

pub fn concat_error_list(es : Vec<ErrorList>) -> ErrorList {
    let mut res = ErrorList::new();
    for e in es.iter().flat_map(|es| es.iter()) {
        res.push(e.clone());
    }
    res
}

pub fn seq_parse_helper<Res>(parse_func : fn(&mut TokenIter) -> Result<Res, ErrorList>,
        it : &mut TokenIter) -> (Option<Res>, ErrorList) {
    let mut tmp = it.clone();
    match parse_func(&mut tmp) {
        Ok(res) => {
            align_iter(it, &mut tmp);
            (Some(res), vec![])
        }
        Err(errs) => (None, errs),
    }
}

pub fn exp_list_to_string<Exp : Display>(exp_list : &Vec<Exp>) -> String {
    if exp_list.len() == 1 {
        return format!("{}", exp_list.first().unwrap());
    }
    let mut it = exp_list.iter();
    let mut s = format!("{}", it.next().unwrap());
    for r in it {
        s.push_str(&format!(", {}", r));
    };
    s
}

pub fn concat_format<Type : Display>(s : String, additional : &str, obj : &Option<Type>) -> String {
    match obj {
        &Some(ref obj) => format!("{} {}{}", s, additional, obj),
        &None => s,
    }
}

pub fn parse_list_helper<Expr>(parse_func : fn(it : &mut TokenIter) -> Result<Expr, ErrorList>,
        it : &mut TokenIter) -> Result<Vec<Expr>, ErrorList> {
    let mut exp_list = vec![];
    exp_list.push(try!(parse_func(it)));
    loop {
        let mut tmp = it.clone();
        if let Err(..) = ::parser::common::consume_next_token_with_type(
                &mut tmp, ::parser::lexer::TokenType::Comma) {
            return Ok(exp_list);
        }
        exp_list.push(try!(parse_func(&mut tmp)));
        align_iter(it, &mut tmp);
    }
}


#[derive(Debug)]
pub enum Statement {
    Select(SelectStatement),
    Update(UpdateStatement),
    Insert(InsertStatement),
    Delete(DeleteStatement),
    Create(CreateStatement),
    Drop(DropStatement),
}

impl Statement {
    pub fn parse(it : &mut TokenIter) -> Result<Statement, ErrorList> {
        let mut tmp = it.clone();
        let type_list = vec![TokenType::Select, TokenType::Update,TokenType::Insert,
            TokenType::Delete, TokenType::Create, TokenType::Drop];
        let token = try!(consume_next_token_with_type_list(&mut tmp, &type_list));
        Ok(match token.token_type {
            TokenType::Select => Statement::Select(try!(SelectStatement::parse(it))),
            TokenType::Update => Statement::Update(try!(UpdateStatement::parse(it))),
            TokenType::Insert => Statement::Insert(try!(InsertStatement::parse(it))),
            TokenType::Delete => Statement::Delete(try!(DeleteStatement::parse(it))),
            TokenType::Create => Statement::Create(try!(CreateStatement::parse(it))),
            TokenType::Drop => Statement::Drop(try!(DropStatement::parse(it))),
            _ => panic!("invalid state"),
        })
    }
}
