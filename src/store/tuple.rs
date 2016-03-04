use std::vec::Vec;
use super::buffer::DataPtr;
use super::table::{AttrType, Attr};


#[derive(Debug)]
pub enum TupleValue {
    Int(i32),
    Float(f32),
    Char(String),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct TupleData {
    attr_data : Vec<DataPtr>,
}

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
