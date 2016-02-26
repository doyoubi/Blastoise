use std::boxed::Box;
use std::option::Option;
use std::fmt::Debug;
use ::store::tuple::TupleData;


// must be object-safe
pub trait ExecIter : Debug {
    fn open(&mut self);
    fn close(&mut self);
    fn get_next(&mut self) -> Option<TupleData>;
    fn explain(&self) -> String;
}

pub type ExecIterRef = Box<ExecIter>;
