use std::vec::Vec;
use super::buffer::DataPtr;
use super::table::AttrType;


#[derive(Debug)]
pub struct TupleDesc {
    attr_desc : Vec<AttrType>,
}

#[derive(Debug)]
pub struct TupleData {
    attr_data : Vec<DataPtr>,
}
