use std::fmt;
use std::fmt::{Formatter, Display};
use std::option::Option::{Some, None};
use std::result::Result::{Ok, Err};
use super::common::{Parser, ParseArithResult, ValueType};
use super::lexer::{TokenIter, TokenType};
use super::compile_error::{ErrorList, CompileErrorType};
use super::common::{
    check_single_token_type,
    parse_single_token_type,
    parse_table_attr,
};


#[derive(Copy, Clone)]
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


#[derive(Copy, Clone)]
enum CmpOp {
    LT,
    GT,
    LE,
    GE,
    EQ,
    NE,
    IS,
    IS_NOT,
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
            &CmpOp::IS => "is".to_string(),
            &CmpOp::IS_NOT => "is not".to_string(),
        })
    }
}


#[derive(Copy, Clone)]
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


fn binary_fmt<T, U>(operator : &U, lhs : &T, rhs : &T, f : &mut Formatter) -> fmt::Result
    where T : Display, U : Display {
    write!(f, "({} {} {})", lhs, operator, rhs)
}

fn unary_fmt<T>(operator: String, operant : &T, f : &mut Formatter) -> fmt::Result
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
            &ConditionExpr::LogicExpr{lhs, rhs, op} => binary_fmt(op, lhs, rhs, f),
            &ConditionExpr::NotExpr{operant} => unary_fmt("not", operant, f),
            &ConditionExpr::CmpExpr{lhs, rhs, op} => binary_fmt(op, lhs, rhs, f),
            &ConditionExpr::ValueExpr(value) => write!(f, "{}", value),
        }
    }
}


pub type ArithRef = Box<ArithExpr>;

pub enum ArithExpr {
    BinaryExpr {
        lhs : ArithRef,
        rhs : ArithRef,
        op : ArithOp,
    },
    MinusExpr { operant : ArithRef },
    ValueExpr { value : String, value_type : super::common::ValueType },
    TableAttr { table : ::store::table::TableRef, attr : ::store::table::AttrRef },
    AggreFuncCall {
        func : String,
        table : ::store::table::TableRef,
        attr : ::store::table::AttrRef,
    },
}

impl Display for ArithExpr {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        match self {
            &ArithExpr::BinaryExpr{lhs, rhs, op} => binary_fmt(op, lhs, rhs, f),
            &ArithExpr::MinusExpr{operant} => unary_fmt("-", operant, f),
            &ArithExpr::ValueExpr{value, valueType} => write!(f, "({}({}))", valueType, value),
            &ArithExpr::TableAttr{table, attr} => write!(f, "({}.{})", table.name, attr.name),
        }
    }
}

impl ArithExpr {
    fn parse(it : &mut TokenIter, parser : & Parser) -> ParseArithResult {
        Err(vec![])
    }

    fn parse_arith_operant(it : &mut TokenIter, parser : &Parser) -> ParseArithResult {
        // if let Some(errs) = check_reach_the_end(it) {
        //     return Err(errs);
        // }
        // let token = it.peekable().peek();
        let token = try!(consume_next_token(it, ))
        match token.token_type {
            TokenType::IntegerLiteral
            | TokenType::FloatLiteral
            | TokenType::StringLiteral
            | TokenType::Null
                => Ok(ArithExpr::ValueExpr{
                        value : token.value.clone(),
                        value_type : token_type_to_value_type(token.token_type)
                    }),
            TokenType::Identifier =>
                parse_table_attr(it, parser),
            _ => (None,
                format!("unexpected tokentype: {:?}, expect Literal or Identifier",
                    token.tokentype))
        }
    }
}

fn token_type_to_value_type(t : TokenType) {
    match t {
        TokenType::IntegerLiteral => ValueType::Integer,
        TokenType::FloatLiteral => ValueType::FloatLiteral,
        TokenType::StringLiteral => ValueType::String,
        TokenType::Null => ValueType::Null,
        _ => panic!("unexpected TokenType: {}", t),
    }
}

fn parse_binary(it : &mut TokenIter, parser : &mut Parser) -> ParseArithResult {
    Err(vec![])
}
