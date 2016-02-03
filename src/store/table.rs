use std::vec::Vec;
use std::rc::Rc;


#[derive(Copy, Clone)]
pub enum AttrType {
    Integer,
    Float,
    String,
}

#[derive(Copy, Clone)]
pub enum AttrProperty {
    Primary,
    Nullable,
}

pub struct Attr {
    name : String,
    attr_type : AttrType,
    properties : Vec<AttrProperty>,
}

pub type AttrRef = Rc<Attr>;

pub struct Table {
    name : String,
    attrs : Vec<Attr>,
}

pub type TableRef = Rc<Table>;
