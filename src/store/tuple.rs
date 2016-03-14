use std::vec::Vec;
use std::ptr::read;
use utils::pointer::read_string;
use super::buffer::DataPtr;
use super::table::{AttrType, Attr};


#[derive(Debug, Clone)]
pub enum TupleValue {
    Int(i32),
    Float(f32),
    Char(String),
}

#[derive(Debug, Clone)]
pub struct TupleDesc {
    pub attr_desc : Vec<AttrType>,
    pub tuple_len : usize,
}

impl TupleDesc {
    pub fn new(attr_list : &Vec<Attr>) -> TupleDesc {
        let attr_desc = attr_list.iter().map(|attr| attr.attr_type.clone()).collect();
        let tuple_len = tuple_len(attr_list);
        TupleDesc{
            attr_desc : attr_desc,
            tuple_len : tuple_len,
        }
    }
}

pub type TupleData = Vec<DataPtr>;

pub fn tuple_len(attr_list : &Vec<Attr>) -> usize {
    let mut l = 0;
    for attr in attr_list {
        l += match attr.attr_type {
            AttrType::Int | AttrType::Float => 4,
            AttrType::Char{len} => (len + 3) / 4 * 4,  // align to 4 bytes
        }
    }
    l
}

pub fn gen_tuple_value(attr_desc : &Vec<AttrType>, tuple_data : TupleData) -> Vec<TupleValue> {
    let mut value_list = Vec::new();
    assert_eq!(attr_desc.len(), tuple_data.len());
    for (attr, p) in attr_desc.iter().zip(tuple_data.iter()) {
        let value = match attr {
            &AttrType::Int => TupleValue::Int(unsafe{read::<i32>(*p as *const i32)}),
            &AttrType::Float => TupleValue::Float(unsafe{read::<f32>(*p as *const f32)}),
            &AttrType::Char{len} => TupleValue::Char(unsafe{read_string(*p, len)}),
        };
        value_list.push(value);
    }
    value_list
}
