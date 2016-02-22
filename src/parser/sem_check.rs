use std::vec::Vec;
use std::collections::HashSet;
use super::attribute::AttributeExpr;
use super::lexer::{Token, TokenRef, TokenType};
use super::compile_error::{CompileError, CompileErrorType, ErrorList, ErrorRef};
use super::common::{Statement, ValueExpr, ValueType};
use super::select::SelectStatement;
use super::update::UpdateStatement;
use super::insert::InsertStatement;
use super::delete::DeleteStatement;
use super::create_drop::{CreateStatement, DropStatement};
use super::condition::{ConditionExpr, ArithExpr, CmpOperantExpr, CmpOp};
use ::store::table::TableSet;


pub type SemResult = Result<(), ErrorList>;


pub fn check_sem(statement : Statement, table_set : &TableSet) -> SemResult {
    match statement {
        Statement::Select(ref stmt) => check_select(stmt, table_set),
        Statement::Update(ref stmt) => check_update(stmt, table_set),
        Statement::Insert(ref stmt) => check_insert(stmt, table_set),
        Statement::Delete(ref stmt) => check_delete(stmt, table_set),
        Statement::Create(ref stmt) => check_create(stmt, table_set),
        Statement::Drop(ref stmt) => check_drop(stmt, table_set),
    }
}

pub fn check_select(stmt : &SelectStatement, table_set : &TableSet) -> SemResult {
    unimplemented!()
}
pub fn check_update(stmt : &UpdateStatement, table_set : &TableSet) -> SemResult {
    unimplemented!()
}
pub fn check_insert(stmt : &InsertStatement, table_set : &TableSet) -> SemResult {
    unimplemented!()
}
pub fn check_delete(stmt : &DeleteStatement, table_set : &TableSet) -> SemResult {
    unimplemented!()
}

pub fn check_condition(condition : &ConditionExpr, table_set : &TableSet, is_having_cond : bool) -> SemResult {
    match condition {
        &ConditionExpr::NotExpr{ref operant} => check_condition(operant, table_set, is_having_cond),
        &ConditionExpr::LogicExpr{ref lhs, ref rhs, .. } => {
            try!(check_condition(lhs, table_set, is_having_cond));
            check_condition(rhs, table_set, is_having_cond)
        }
        &ConditionExpr::CmpExpr{ref lhs, ref rhs, op } => {
            match op {
                CmpOp::LT | CmpOp::GT | CmpOp::LE | CmpOp::GE => {
                    match (lhs.get_type(), rhs.get_type()) {
                        (ValueType::String, _) | (ValueType::Null, _)
                        | (_, ValueType::String) | (_, ValueType::Null) => {
                                return Err(create_error(CompileErrorType::SemInvalidValueType,
                                    format!("invalid operant type: {} {} {}", lhs, rhs, op)))
                            }
                        _ => ()
                    }
                }
                CmpOp::EQ | CmpOp::NE => {
                    match (lhs.get_type(), rhs.get_type()) {
                        (ValueType::Null, _) | (_, ValueType::Null)  => {
                            return Err(create_error(CompileErrorType::SemInvalidValueType,
                                format!("invalid operant type: {} {} {}", lhs, rhs, op)))
                        }
                        _ => ()
                    }
                }
                CmpOp::Is | CmpOp::IsNot => {
                    match lhs {
                        &CmpOperantExpr::Arith(ArithExpr::Attr(..)) => (),
                        _ => return Err(create_error(CompileErrorType::SemInvalidValueType,
                            format!("expected attribute or aggregate function\
                                in the left of `is` and `is not`, found {}", lhs)))
                    }
                    match rhs {
                        &CmpOperantExpr::Value(ValueExpr{value_type : ValueType::Null, ..}) => (),
                        _ => return Err(create_error(CompileErrorType::SemInvalidValueType,
                            format!("only null is allowd after `is` or `is not`, found {}", rhs)))
                    }
                }
            }
            if let &CmpOperantExpr::Arith(ref lhs_arith) = lhs {
                try!(check_arith_expr(lhs_arith, table_set, is_having_cond));
            }
            if let &CmpOperantExpr::Arith(ref rhs_arith) = rhs {
                try!(check_arith_expr(rhs_arith, table_set, is_having_cond));
            }
            Ok(())
        }
    }
}

pub fn check_arith_expr(arith : &ArithExpr, table_set : &TableSet, is_having_cond : bool) -> SemResult {
    match arith {
        &ArithExpr::Value(ValueExpr{value_type, ..}) => {
            // already guranteed by grammar
            assert!(value_type == ValueType::Integer || value_type == ValueType::Float);
            Ok(())
        }
        &ArithExpr::MinusExpr{ref operant} => check_arith_expr(operant, table_set, is_having_cond),
        &ArithExpr::BinaryExpr{ref lhs, ref rhs, ..} => {
            try!(check_arith_expr(lhs, table_set, is_having_cond));
            check_arith_expr(rhs, table_set, is_having_cond)
        }
        &ArithExpr::Attr(ref attr) => check_attr(attr, table_set, is_having_cond),
    }
}

pub fn check_attr(attr : &AttributeExpr, table_set : &TableSet, is_having_cond : bool) -> SemResult {
    match attr {
        &AttributeExpr::TableAttr{ref table, ref attr} => Ok(()),
        &AttributeExpr::AggreFuncCall{ref func, ref table, ref attr} => Ok(()),
    }
}

pub fn check_attr_exist(table : &str, attr : &str, table_set : &TableSet) {
}

pub fn check_create(stmt : &CreateStatement, table_set : &TableSet) -> SemResult {
    try!(check_create_table_exit(stmt, table_set));
    try!(check_unique_primary(stmt));
    try!(check_primary_not_null(stmt));
    try!(check_attr_unique(stmt));
    Ok(())
}

pub fn check_create_table_exit(stmt : &CreateStatement, table_set : &TableSet) -> SemResult {
    if table_set.exist(&stmt.table) {
        Err(vec![ErrorRef::new(CompileError{
            error_type : CompileErrorType::SemTableExist,
            token : dummy_token(),
            error_msg : format!("table {} already exist", &stmt.table),
        })])
    } else {
        Ok(())
    }
}

pub fn check_unique_primary(stmt : &CreateStatement) -> SemResult {
    let primary_attr_list : Vec<String> =
        stmt.decl_list.iter().filter(|d| d.primary).map(|d| d.name.clone()).collect();
    if primary_attr_list.len() == 1 {
        Ok(())
    } else if primary_attr_list.is_empty() {
        Err(vec![ErrorRef::new(CompileError{
            error_type : CompileErrorType::SemNoPrimary,
            token : dummy_token(),
            error_msg : "no primary attribute found".to_string(),
        })])
    } else {
        Err(vec![ErrorRef::new(CompileError{
            error_type : CompileErrorType::SemMultiplePrimary,
            token : dummy_token(),
            error_msg : format!("multiple primary not support: {:?}", primary_attr_list),
        })])
    }
}

pub fn check_primary_not_null(stmt : &CreateStatement) -> SemResult {
    for decl in stmt.decl_list.iter().filter(|d| d.primary && d.nullable) {
        return Err(vec![ErrorRef::new(CompileError{
            error_type : CompileErrorType::SemNullablePrimary,
            token : dummy_token(),
            error_msg : format!("primary attribute can't be null: {}", decl.name),
        })]);
    }
    Ok(())
}

pub fn check_attr_unique(stmt : &CreateStatement) -> SemResult {
    let mut table_set = HashSet::new();
    for name in stmt.decl_list.iter().map(|d| &d.name) {
        if table_set.contains(name) {
            return Err(vec![ErrorRef::new(CompileError{
                error_type : CompileErrorType::SemDuplicateAttr,
                token : dummy_token(),
                error_msg : format!("duplicate attribute name :{}", name),
            })])
        } else {
            table_set.insert(name);
        }
    }
    Ok(())
}

pub fn check_drop(stmt : &DropStatement, table_set : &TableSet) -> SemResult {
    check_table_exist(&stmt.table, table_set)
}

pub fn check_table_exist(table : &str, table_set : &TableSet) -> SemResult {
    if table_set.exist(table) {
        Ok(())
    } else {
        Err(vec![ErrorRef::new(CompileError{
            error_type : CompileErrorType::SemTableNotExist,
            token : dummy_token(),
            error_msg : table_not_exist(table),
        })])
    }
}

pub fn table_not_exist(table : &str) -> String {
    format!("table `{}` not exist", table)
}

pub fn dummy_token() -> TokenRef {
    TokenRef::new(Token{
        column : 0,
        value : "".to_string(),
        token_type : TokenType::UnKnown
    })
}

pub fn create_error(error_type : CompileErrorType, error_msg : String) -> ErrorList {
    vec![ErrorRef::new(CompileError{
            error_type : error_type,
            token : dummy_token(),
            error_msg : error_msg,
        })]
}
