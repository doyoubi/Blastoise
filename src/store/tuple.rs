use std::vec::Vec;
use super::buffer::DataPtr;
use super::table::{AttrType, Attr};


#[derive(Debug)]
pub struct TupleDesc {
    attr_desc : Vec<AttrType>,
}

#[derive(Debug)]
pub struct TupleData {
    attr_data : Vec<DataPtr>,
}

pub fn tuple_len(attr_list : &Vec<Attr>) -> usize {
    let mut len = 0;
    for attr in attr_list {
        len += match attr.attr_type {
            AttrType::Int | AttrType::Float => 4,
            AttrType::Char{len} => len,
        }
    }
    len
}
