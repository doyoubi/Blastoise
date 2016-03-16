use super::sem_check::dummy_token;
use super::common::ValueType;
use super::compile_error::{CompileError, CompileErrorType, ErrorList, ErrorRef};
use super::condition::{ConditionExpr, ArithExpr, CmpOperantExpr};


pub type UnimplResult = Result<(), ErrorList>;


pub fn check_cond(condition : &ConditionExpr) -> UnimplResult {
    match condition {
        &ConditionExpr::NotExpr{ref operant} => check_cond(operant),
        &ConditionExpr::LogicExpr{ref lhs, ref rhs, .. } => {
            try!(check_cond(lhs));
            check_cond(rhs)
        }
        &ConditionExpr::CmpExpr{ref lhs, ref rhs, .. } => {
            try!(check_cmp_operant(lhs));
            check_cmp_operant(rhs)
        }
    }
}

pub fn check_cmp_operant(operant : &CmpOperantExpr) -> UnimplResult {
    match operant {
        &CmpOperantExpr::Value(ref value) => {
            if value.value_type == ValueType::Null {
                return Err(gen_null_error())
            } else {
                Ok(())
            }
        }
        &CmpOperantExpr::Arith(ref arith) => {
            check_arith_operant(arith)
        }
    }
}

pub fn check_arith_operant(arith : &ArithExpr) -> UnimplResult {
    match arith {
        &ArithExpr::BinaryExpr{ref lhs, ref rhs, ..} => {
            try!(check_arith_operant(lhs));
            check_arith_operant(rhs)
        }
        &ArithExpr::MinusExpr{ref operant} => {
            check_arith_operant(operant)
        }
        &ArithExpr::Value(ref value) => {
            if value.value_type == ValueType::Null {
                return Err(gen_null_error())
            } else {
                Ok(())
            }
        }
        &ArithExpr::Attr(..) => Ok(()),
    }
}

fn gen_null_error() -> ErrorList {
    vec![ErrorRef::new(CompileError{
            error_type : CompileErrorType::SemUnimplemented,
            token : dummy_token(),
            error_msg : "null not supported".to_string(),
        })]
}
