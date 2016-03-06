use std::ptr::read;
use std::collections::HashMap;
use ::parser::condition::{
    ConditionExpr,
    ArithExpr,
    ArithOp,
};
use ::parser::common::{ValueExpr, ValueType};
use ::parser::attribute::AttributeExpr;
use ::store::buffer::DataPtr;


pub type PtrMap = HashMap<(String, String), (DataPtr, ValueType)>;


pub fn evaluate(_condition : &ConditionExpr, _ptr_map : &PtrMap) -> bool {
    false
}

pub fn eval_arith(expr : &ArithExpr, ptr_map : &PtrMap) -> f32 {
    match expr {
        &ArithExpr::BinaryExpr{ ref lhs, ref rhs, op } => {
            let l = eval_arith(lhs, ptr_map);
            let r = eval_arith(rhs, ptr_map);
            match op {
                ArithOp::Add => l + r,
                ArithOp::Sub => l - r,
                ArithOp::Mul => l * r,
                ArithOp::Div => l / r,
                ArithOp::Mod => l % r,
            }
        }
        &ArithExpr::MinusExpr{ ref operant } => - eval_arith(operant, ptr_map),
        &ArithExpr::Value(ref v) => eval_num(v),
        &ArithExpr::Attr( ref attr_expr ) => {
            let (table, attr) = match attr_expr {
                &AttributeExpr::TableAttr{ref table, ref attr} => (table.clone(), attr.clone()),
                &AttributeExpr::AggreFuncCall{ref table, ref attr, ..} => (table.clone(), attr.clone()),
            };
            let (p, t) = ptr_map.get(&(table.unwrap(), attr)).unwrap().clone();
            match t {
                ValueType::Integer => unsafe{ read::<i32>(p as *const i32) as f32 },
                ValueType::Float => unsafe{ read::<f32>(p as *const f32) },
                _ => panic!("invalid type {:?}", t),
            }
        }
    }
}

pub fn eval_num(expr : &ValueExpr) -> f32 {
    match expr.value_type {
        ValueType::Integer => expr.value.parse::<i32>().unwrap() as f32,
        ValueType::Float => expr.value.parse::<f32>().unwrap(),
        t => panic!("invalid type {:?}", t),
    }
}
