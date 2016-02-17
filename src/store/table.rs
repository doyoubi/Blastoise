use std::vec::Vec;
use std::rc::Rc;


#[derive(Copy, Clone)]
pub enum AttrType {
    Int,
    Float,
    Char{ len : usize },
}

#[derive(Copy, Clone)]
pub enum AttrProperty {
    Primary,
    Nullable,
}

pub struct Attr {
    pub name : String,
    pub attr_type : AttrType,
    pub properties : Vec<AttrProperty>,
}

pub type AttrRef = Rc<Attr>;

pub struct Table {
    pub name : String,
    pub attrs : Vec<AttrRef>,
}

pub type TableRef = Rc<Table>;
