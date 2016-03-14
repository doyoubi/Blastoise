use std::vec::Vec;
use std::clone::Clone;


pub fn projection<T : Clone>(proj_index : &Vec<usize>, source : Vec<T>) -> Vec<T> {
    let mut res = Vec::new();
    for i in proj_index.iter() {
        res.push(source[*i].clone());
    }
    res
}
