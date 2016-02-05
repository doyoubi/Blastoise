use std::fmt;
use std::fmt::{Formatter, Display};
use std::rc::Rc;
use std::option::Option::{Some, None};
use std::result::Result::{Ok, Err};
use super::common::{ParseArithResult, ValueType};
use super::lexer::{TokenIter, TokenType};
use super::compile_error::{CompileError, CompileErrorType};
use super::common::{
    consume_next_token,
    parse_table_attr,
};


#[derive(Copy, Clone, Debug)]
enum LogicOp {
    Or,
    And,
}

impl Display for LogicOp {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            &LogicOp::Or => "or".to_string(),
            &LogicOp::And => "and".to_string(),
        })
    }
}


#[derive(Copy, Clone, Debug)]
enum CmpOp {
    LT,
    GT,
    LE,
    GE,
    EQ,
    NE,
    Is,
    IsNot,
}

impl Display for CmpOp {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            &CmpOp::LT => "<".to_string(),
            &CmpOp::GT => ">".to_string(),
            &CmpOp::LE => "<=".to_string(),
            &CmpOp::GE => ">=".to_string(),
            &CmpOp::EQ => "=".to_string(),
            &CmpOp::NE => "!=".to_string(),
            &CmpOp::Is => "is".to_string(),
            &CmpOp::IsNot => "is not".to_string(),
        })
    }
}


#[derive(Copy, Clone, Debug)]
enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl Display for ArithOp {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            &ArithOp::Add => "+".to_string(),
            &ArithOp::Sub => "-".to_string(),
            &ArithOp::Mul => "*".to_string(),
            &ArithOp::Div => "/".to_string(),
            &ArithOp::Mod => "%".to_string(),
        })
    }
}


trait Expr : Display + ToString {}


fn binary_fmt<T, U>(operator : U, lhs : &T, rhs : &T, f : &mut Formatter) -> fmt::Result
    where T : Display, U : Display {
    write!(f, "({} {} {})", lhs, operator, rhs)
}

fn unary_fmt<T>(operator: &str, operant : &T, f : &mut Formatter) -> fmt::Result
    where T : Display {
    write!(f, "({} {})", operator, operant)
}

type CondRef = Box<ConditionExpr>;

enum ConditionExpr {
    LogicExpr {
        lhs : CondRef,
        rhs :CondRef,
        op : LogicOp,
    },
    NotExpr { operant : CondRef },
    CmpExpr {
        lhs : ArithRef,
        rhs : ArithRef,
        op : CmpOp,
    },
    ValueExpr(bool),
}

impl Display for ConditionExpr {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        match self {
            &ConditionExpr::LogicExpr{ref lhs, ref rhs, op} => binary_fmt(op, lhs, rhs, f),
            &ConditionExpr::NotExpr{ref operant} => unary_fmt("not", operant, f),
            &ConditionExpr::CmpExpr{ref lhs, ref rhs, op} => binary_fmt(op, lhs, rhs, f),
            &ConditionExpr::ValueExpr(value) => write!(f, "{}", value),
        }
    }
}


pub type ArithRef = Box<ArithExpr>;

#[derive(Debug)]
pub enum ArithExpr {
    BinaryExpr {
        lhs : ArithRef,
        rhs : ArithRef,
        op : ArithOp,
    },
    MinusExpr { operant : ArithRef },
    ValueExpr { value : String, value_type : super::common::ValueType },
    TableAttr { table : Option<String>, attr : String },
    AggreFuncCall {
        func : String,
        table : String,
        attr : String,
    },
}

impl Display for ArithExpr {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        match self {
            &ArithExpr::BinaryExpr{ref lhs, ref rhs, op} => binary_fmt(op, lhs, rhs, f),
            &ArithExpr::MinusExpr{ref operant} => unary_fmt("-", operant, f),
            &ArithExpr::ValueExpr{ref value, value_type} => write!(f, "({:?}({}))", value_type, value),
            &ArithExpr::TableAttr{ref table, ref attr} => {
                match table {
                    &Some(ref table) => write!(f, "({}.{})", table, attr),
                    &None => write!(f, "{}", attr),
                }
            }
            &ArithExpr::AggreFuncCall{ref func, ref table, ref attr} =>
                write!(f, "{}({}.{})", func, table, attr),
        }
    }
}

impl ArithExpr {
    pub fn parse(it : &mut TokenIter) -> ParseArithResult {
        Err(vec![])
    }

    pub fn parse_arith_operant(it : &mut TokenIter) -> ParseArithResult {
        let token = try!(consume_next_token(it));
        match token.token_type {
            TokenType::IntegerLiteral
            | TokenType::FloatLiteral
            | TokenType::StringLiteral
            | TokenType::Null
                => Ok(ArithExpr::ValueExpr{
                        value : token.value.clone(),
                        value_type : token_type_to_value_type(token.token_type),
                    }),
            TokenType::Identifier =>
                parse_table_attr(it),
            _ => {
                let err_msg = format!("unexpected tokentype: {:?}, expect Literal or Identifier",
                    token.token_type);
                let e = Rc::new(CompileError{
                    error_type : CompileErrorType::ParserUnExpectedTokenType,
                    token : token,
                    error_msg : err_msg,
                });
                Err(vec![e])
            }
        }
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

fn parse_binary(it : &mut TokenIter) -> ParseArithResult {
    Err(vec![])
}
