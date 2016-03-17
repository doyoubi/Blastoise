use super::sem_check::dummy_token;
use super::common::{ValueType, Statement};
use super::attribute::AttributeExpr;
use super::compile_error::{CompileError, CompileErrorType, ErrorList, ErrorRef};
use super::condition::{ConditionExpr, ArithExpr, CmpOperantExpr};
use super::select::{SelectStatement, SelectExpr, Relation};


pub type UnimplResult = Result<(), ErrorList>;



macro_rules! check_stmt_cond {
    ($stmt:expr) => ({
        match $stmt.where_condition {
            Some(ref cond) => check_cond(cond),
            None => Ok(()),
        }
    })
}


pub fn check_stmt_unimpl(stmt : &Statement) -> UnimplResult {
    match stmt {
        &Statement::Select(ref select) => check_select(select),
        &Statement::Delete(ref delete) => check_stmt_cond!(&delete),
        &Statement::Update(ref update) => check_stmt_cond!(&update),
        _ => Ok(())
    }
}


pub fn check_select(select : &SelectStatement) -> UnimplResult {
    if select.groupby_having.is_some() {
        return Err(gen_unimpl_error("group by and having not supported"));
    }
    if select.order_by_attr.is_some() {
        return Err(gen_unimpl_error("order by not supported"));
    }
    if let SelectExpr::AttrList(ref attr_list) = select.select_expr {
        for attr in attr_list.iter() {
            if let &AttributeExpr::AggreFuncCall{..} = attr {
                return Err(gen_unimpl_error("aggregate function not supported"));
            }
        }
    }
    if select.relation_list.len() > 1 {
        return Err(gen_unimpl_error("select from multiple tables not supported"));
    }
    for r in select.relation_list.iter() {
        if let &Relation::Select(..) = r {
            return Err(gen_unimpl_error("sub query not supported"));
        }
    }
    if let Some(ref cond) = select.where_condition {
        try!(check_cond(cond));
    }
    Ok(())
}

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
    gen_unimpl_error("null not supported")
}

fn gen_unimpl_error(err_msg : &str) -> ErrorList {
    vec![ErrorRef::new(CompileError{
            error_type : CompileErrorType::SemUnimplemented,
            token : dummy_token(),
            error_msg : err_msg.to_string(),
        })]
}
