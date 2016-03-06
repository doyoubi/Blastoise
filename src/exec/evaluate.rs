use std::ptr::read;
use std::collections::HashMap;
use ::parser::condition::{
    ConditionExpr,
    ArithExpr,
    ArithOp,
    CmpOperantExpr,
    CmpOp,
    CondRef,
    LogicOp,
};
use ::parser::common::{ValueExpr, ValueType};
use ::parser::attribute::AttributeExpr;
use ::store::buffer::DataPtr;
use ::store::table::AttrType;
use ::utils::pointer::read_string;


pub type PtrMap = HashMap<(String, String), (DataPtr, AttrType)>;


pub fn eval_cond(condition : &ConditionExpr, ptr_map : &PtrMap) -> bool {
    match condition {
        &ConditionExpr::NotExpr{ ref operant } => !eval_cond(operant, ptr_map),
        &ConditionExpr::CmpExpr{ ref lhs, ref rhs, op } =>
            eval_cmp_operant(lhs, rhs, op, ptr_map),
        &ConditionExpr::LogicExpr{ ref lhs, ref rhs, op } =>
            eval_logic_op(lhs, rhs, op, ptr_map),
    }
}

pub fn eval_logic_op(lhs : &CondRef, rhs : &CondRef, op : LogicOp, ptr_map : &PtrMap) -> bool {
    let lresult = eval_cond(&**lhs, ptr_map);
    let rresult = eval_cond(&**rhs, ptr_map);
    match op {
        LogicOp::Or => lresult || rresult,
        LogicOp::And => lresult && rresult,
    }
}

pub fn eval_cmp_operant(
        lhs : &CmpOperantExpr,
        rhs : &CmpOperantExpr,
        op : CmpOp,
        ptr_map : &PtrMap) -> bool {
    match (lhs, rhs) {
        (&CmpOperantExpr::Value(ref l), &CmpOperantExpr::Value(ref r)) => {
            let lvalue = eval_str(l);
            let rvalue = eval_str(r);
            eval_str_cmp(&lvalue, &rvalue, op)
        }
        (&CmpOperantExpr::Value(ref l), &CmpOperantExpr::Arith(ref r)) => {
            let lvalue = eval_str(l);
            let rvalue = eval_str_attr(r, ptr_map);
            eval_str_cmp(&lvalue, &rvalue, op)
        }
        (&CmpOperantExpr::Arith(ref l), &CmpOperantExpr::Value(ref r)) => {
            let lvalue = eval_str_attr(l, ptr_map);
            let rvalue = eval_str(r);
            eval_str_cmp(&lvalue, &rvalue, op)
        }
        (&CmpOperantExpr::Arith(ref l), &CmpOperantExpr::Arith(ref r)) => {
            let lvalue = eval_arith(l, ptr_map);
            let rvalue = eval_arith(r, ptr_map);
            match op {
                CmpOp::LT => lvalue < rvalue,
                CmpOp::GT => lvalue > rvalue,
                CmpOp::LE => lvalue <= rvalue,
                CmpOp::GE => lvalue >= rvalue,
                CmpOp::EQ => lvalue == rvalue,
                CmpOp::NE => lvalue != rvalue,
                CmpOp::Is => unimplemented!(),
                CmpOp::IsNot => unimplemented!(),
            }
        }
    }
}

pub fn eval_str_cmp(lvalue : &String, rvalue : &String, op : CmpOp) -> bool {
    match op {
        CmpOp::LT | CmpOp::GT| CmpOp::LE| CmpOp::GE =>
            panic!("invalid operationo for string"),
        CmpOp::EQ => lvalue == rvalue,
        CmpOp::NE => lvalue != rvalue,
        CmpOp::Is => unimplemented!(),
        CmpOp::IsNot => unimplemented!(),
    }
}

pub fn eval_str_attr(expr : &ArithExpr, ptr_map : &PtrMap) -> String {
    match expr {
        &ArithExpr::Attr( ref attr_expr ) => {
            let (table, attr) = match attr_expr {
                &AttributeExpr::TableAttr{ref table, ref attr} => (table.clone(), attr.clone()),
                &AttributeExpr::AggreFuncCall{ref table, ref attr, ..} => (table.clone(), attr.clone()),
            };
            assert!(table.is_some());
            let (p, t) = ptr_map.get(&(table.unwrap(), attr)).unwrap().clone();
            let len = extract!(t, AttrType::Char{len}, len);
            unsafe{ read_string(p, len) }
        }
        _ => panic!("expected attribute, found {:?}", expr),
    }
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
                AttrType::Int => unsafe{ read::<i32>(p as *const i32) as f32 },
                AttrType::Float => unsafe{ read::<f32>(p as *const f32) },
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

pub fn eval_str(expr : &ValueExpr) -> String {
    match expr.value_type {
        ValueType::String => expr.value.clone(),
        t => panic!("invalid type {:?}", t),
    }
}
