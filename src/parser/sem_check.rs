use std::vec::Vec;
use std::collections::HashSet;
use super::attribute::AttributeExpr;
use super::lexer::{Token, TokenRef, TokenType};
use super::compile_error::{CompileError, CompileErrorType, ErrorList, ErrorRef};
use super::common::{Statement, ValueExpr, ValueType};
use super::select::{SelectStatement, GroupbyHaving, SelectExpr};
use super::update::UpdateStatement;
use super::insert::InsertStatement;
use super::delete::DeleteStatement;
use super::create_drop::{CreateStatement, DropStatement};
use super::condition::{ConditionExpr, ArithExpr, CmpOperantExpr, CmpOp};
use ::store::table::{TableSet, AttrType, Attr};


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
    if let Some(ref cond) = stmt.where_condition {
        try!(check_condition(cond, table_set, &None));
    }
    if let Some(GroupbyHaving{ref attr, ref having_condition}) = stmt.groupby_having {
        let (table, attr) = attr.get_attr();
        try!(check_attr_exist(table, attr, table_set));
        let group_by_attr = Some((table.clone(), attr.clone()));
        if let &Some(ref cond) = having_condition {
            try!(check_condition(cond, table_set, &group_by_attr));
        }
        match stmt.select_expr {
            SelectExpr::AllAttribute =>
                return Err(create_error(CompileErrorType::SemSelectAllWithGroupBy,
                    "can't select all when using group by".to_string())),
            SelectExpr::AttrList(ref attr_list) => {
                for attr_expr in attr_list {
                    try!(check_attr(attr_expr, table_set, &group_by_attr));
                }
            }
        }
        if let Some(ref attr) = stmt.order_by_attr {
            try!(check_attr(attr, table_set, &group_by_attr));
        }
    } else {
        if let SelectExpr::AttrList(ref attr_list) = stmt.select_expr {
            for attr_expr in attr_list {
                try!(check_attr(attr_expr, table_set, &None));
            }
        }
        if let Some(ref attr) = stmt.order_by_attr {
            try!(check_attr(attr, table_set, &None));
        }
    }
    Ok(())
}

pub fn check_update(stmt : &UpdateStatement, table_set : &TableSet) -> SemResult {
    try!(check_table_exist(&stmt.table, table_set));
    if let Some(ref cond) = stmt.where_condition {
        try!(check_condition(cond, table_set, &None));
    }
    for assign in &stmt.set_list {
        try!(check_attr_exist(&Some(stmt.table.clone()), &assign.attr, table_set));
        let attr = table_set.get_attr(&Some(stmt.table.clone()), &assign.attr).unwrap();
        if attr.primary {
            return Err(create_error(CompileErrorType::SemChangePrimaryAttr,
                format!("can't change primary attribute: {}", attr.name)));
        }
        try!(check_assign(&assign.value, &attr));
    }
    Ok(())
}

pub fn check_insert(stmt : &InsertStatement, table_set : &TableSet) -> SemResult {
    try!(check_table_exist(&stmt.table, table_set));
    let value_list  = &stmt.value_list;
    let attr_list = table_set.gen_attr_list(&stmt.table);  // table should exist
    if attr_list.len() != value_list.len() {
        return Err(create_error(CompileErrorType::SemInvalidInsertValuesNum,
            format!("invalid insert values number, expected {}, found {}",
                attr_list.len(), value_list.len())));
    }
    for (value, attr) in value_list.iter().zip(attr_list.iter()) {
        try!(check_assign(value, attr));
    }
    Ok(())
}

pub fn check_assign(value : &ValueExpr, attr : &Attr) -> SemResult {
    match (value.value_type, attr.attr_type) {
            (ValueType::Integer, AttrType::Int)
        | (ValueType::Integer, AttrType::Float)
        | (ValueType::Float, AttrType::Float) => (),
        (ValueType::String, AttrType::Char{len}) => {
            if value.value.len() > len {
                return Err(create_error(CompileErrorType::SemInvalidInsertCharLen,
                    format!("invalid char len, expected {}, found {}", len, value.value.len())));
            }
        }
        (ValueType::Null, _) => {
            if !attr.nullable {
                return Err(create_error(CompileErrorType::SemAttributeNotNullable,
                    format!("attribute {} is not nullable", attr.name)));
            }
        }
        (value_type, attr_type) =>
            return Err(create_error(CompileErrorType::SemInvalidInsertValueType,
                format!("invalid insert value type, attribute type is {:?}, found {:?}",
                    attr_type, value_type))),
    }
    Ok(())
}

pub fn check_delete(stmt : &DeleteStatement, table_set : &TableSet) -> SemResult {
    try!(check_table_exist(&stmt.table, table_set));
    match &stmt.where_condition {
        &Some(ref cond) => check_condition(cond, table_set, &None),
        &None => Ok(()),
    }
}

pub fn check_condition(
        condition : &ConditionExpr,
        table_set : &TableSet,
        group_by_attr : &Option<(Option<String>, String)>) -> SemResult {
    match condition {
        &ConditionExpr::NotExpr{ref operant} => check_condition(operant, table_set, &group_by_attr),
        &ConditionExpr::LogicExpr{ref lhs, ref rhs, .. } => {
            try!(check_condition(lhs, table_set, &group_by_attr));
            check_condition(rhs, table_set, &group_by_attr)
        }
        &ConditionExpr::CmpExpr{ref lhs, ref rhs, op } => {
            let must_be_num_type = match op {
                CmpOp::LT | CmpOp::GT | CmpOp::LE | CmpOp::GE => {
                    match (lhs.get_type(), rhs.get_type()) {
                        (ValueType::String, _) | (ValueType::Null, _)
                        | (_, ValueType::String) | (_, ValueType::Null) => {
                                return Err(create_error(CompileErrorType::SemInvalidValueType,
                                    format!("invalid operant type: {} {} {}", lhs, rhs, op)))
                            }
                        _ => ()
                    }
                    true
                }
                CmpOp::EQ | CmpOp::NE => {
                    match (lhs.get_type(), rhs.get_type()) {
                        (ValueType::Null, _) | (_, ValueType::Null)  => {
                            return Err(create_error(CompileErrorType::SemInvalidValueType,
                                format!("invalid operant type: {} {} {}", lhs, rhs, op)))
                        }
                        _ => ()
                    }
                    false
                }
                CmpOp::Is | CmpOp::IsNot => {
                    match lhs {
                        &CmpOperantExpr::Arith(ArithExpr::Attr(ref attr)) => {
                            try!(check_is_nullable(attr, table_set));
                        }
                        _ => return Err(create_error(CompileErrorType::SemInvalidValueType,
                            format!("expected attribute or aggregate function\
                                in the left of `is` and `is not`, found {}", lhs)))
                    }
                    match rhs {
                        &CmpOperantExpr::Value(ValueExpr{value_type : ValueType::Null, ..}) => (),
                        _ => return Err(create_error(CompileErrorType::SemInvalidValueType,
                            format!("only null is allowd after `is` or `is not`, found {}", rhs)))
                    }
                    false
                }
            };
            if let &CmpOperantExpr::Arith(ref lhs_arith) = lhs {
                try!(check_arith_expr(lhs_arith, table_set, must_be_num_type, &group_by_attr));
            }
            if let &CmpOperantExpr::Arith(ref rhs_arith) = rhs {
                try!(check_arith_expr(rhs_arith, table_set, must_be_num_type, &group_by_attr));
            }
            Ok(())
        }
    }
}

pub fn check_is_nullable(attr_expr : &AttributeExpr, table_set : &TableSet) -> SemResult {
    let (table, attr) = attr_expr.get_attr();
    try!(check_attr_exist(table, attr, table_set));
    if !table_set.get_attr(table, attr).unwrap().nullable {
        return Err(create_error(CompileErrorType::SemAttributeNotNullable,
            format!("attribute `{}` is not nullale", attr)));
    }
    Ok(())
}

pub fn check_arith_expr(
        arith : &ArithExpr,
        table_set : &TableSet,
        must_be_num_type : bool,
        group_by_attr : &Option<(Option<String>, String)>) -> SemResult {
    match arith {
        &ArithExpr::Value(ValueExpr{value_type, ..}) => {
            // already guranteed by grammar
            assert!(value_type == ValueType::Integer || value_type == ValueType::Float);
            Ok(())
        }
        &ArithExpr::MinusExpr{ref operant} => {
            check_arith_expr(operant, table_set, must_be_num_type, &group_by_attr)
        }
        &ArithExpr::BinaryExpr{ref lhs, ref rhs, ..} => {
            try!(check_arith_expr(lhs, table_set, must_be_num_type, &group_by_attr));
            check_arith_expr(rhs, table_set, must_be_num_type, &group_by_attr)
        }
        &ArithExpr::Attr(ref attr) => {
            try!(check_attr(attr, table_set, &group_by_attr));
            if must_be_num_type {
                check_attr_num_type(attr, table_set)
            } else {
                Ok(())
            }
        }
    }
}

pub fn check_attr_num_type(attr_expr : &AttributeExpr, table_set : &TableSet) -> SemResult {
    let (table, attr) = attr_expr.get_attr();
    let attr = table_set.get_attr(table, attr).unwrap();
    if let AttrType::Char{..} = attr.attr_type {
        return Err(create_error(CompileErrorType::SemInvalidValueType,
            format!("invalid attribute type: {}", attr_expr)));
    }
    Ok(())
}

pub fn check_attr(
        attr_expr : &AttributeExpr,
        table_set : &TableSet,
        group_by_attr : &Option<(Option<String>, String)>) -> SemResult {
    let (table, attr) = match attr_expr {
        &AttributeExpr::TableAttr{ref table, ref attr} => {
            try!(check_attr_exist(table, attr, table_set));
            (table, attr)
        }
        &AttributeExpr::AggreFuncCall{ref func, ref table, ref attr} => {
            try!(check_aggre_func_name(func));
            try!(check_attr_exist(table, attr, table_set));
            if let &None = group_by_attr {
                return Err(create_error(CompileErrorType::SemInvalidAggregateFunctionUse,
                    format!("can't use {} in `where`", attr_expr)));
            }
            (table, attr)
        }
    };
    let group_by_attr = match group_by_attr {
        &Some(ref expr) => expr,
        &None => return Ok(()),
    };
    // unique already guranteed
    if (!is_match!(group_by_attr.0, None) && !is_match!(table, &None) && group_by_attr.0 != *table)
            || group_by_attr.1 != *attr {
        return Err(create_error(CompileErrorType::SemShouldUseGroupByAttribute,
            format!("expected group by attribute {:?}, got {}", group_by_attr, attr_expr)))
    }
    Ok(())
}

pub fn check_attr_exist(table : &Option<String>, attr : &str, table_set : &TableSet) -> SemResult {
    if table_set.get_attr(table, attr).is_some() {
        Ok(())
    } else {
        Err(create_error(CompileErrorType::SemInvalidAttribute,
            format!("{} not exist or multiple found", attr)))
    }
}

pub fn check_aggre_func_name(name : &String) -> SemResult {
    let aggre_func_list = ["max", "min", "count", "sum"];
    if aggre_func_list.into_iter().filter(|s| *name == s.to_string()).next().is_some() {
        Ok(())
    } else {
        Err(create_error(CompileErrorType::SemInvalidAggreFuncName,
            format!("invalid aggregate function name: {}, expected {:?}", name, aggre_func_list)))
    }
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
