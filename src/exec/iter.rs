use std::boxed::Box;
use std::option::Option;
use std::fmt::Debug;
use ::store::tuple::TupleData;
use super::error::ExecError;


// must be object-safe
pub trait ExecIter : Debug {
    fn open(&mut self);
    fn close(&mut self);
    fn get_next(&mut self) -> Option<TupleData>;
    fn explain(&self) -> String;
    fn get_error(&self) -> Option<ExecError>;
}

pub type ExecIterRef = Box<ExecIter>;
