use std::vec::Vec;
use std::collections::{BTreeMap, HashMap};
use std::option::Option;
use std::rc::Rc;
use std::cell::RefCell;
use rustc_serialize::{Encodable, Decodable, Encoder, Decoder};
use rustc_serialize::json::{encode, decode};
use super::tuple::TupleDesc;


macro_rules! unwrap {
    ($result:expr) => ({
        match $result {
            Err(err) => panic!("unexpected error: {:?}", err),
            Ok(data) => data,
        }
    })
}

#[derive(Debug, Copy, Clone)]
pub enum AttrType {
    Int,
    Float,
    Char{ len : usize },
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Attr {
    pub name : String,
    pub attr_type : AttrType,
    pub primary : bool,
    pub nullable : bool,
}


pub type TableRef = Rc<RefCell<Table>>;

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Table {
    pub name : String,
    pub attr_list : Vec<Attr>,
}

impl Table {
    pub fn gen_tuple_desc(&self) -> TupleDesc {
        TupleDesc::new(&self.attr_list)
    }
}


pub struct TableSet {
    pub tables : HashMap<String, Table>,
}

impl TableSet {
    pub fn new() -> TableSet {
        TableSet{
            tables : HashMap::new(),
        }
    }
    pub fn exist(&self, name : &str) -> bool {
        match self.tables.get(name) {
            Some(..) => true,
            None => false,
        }
    }
    pub fn get_attr(&self, table : &Option<String>, attr : &str) -> Option<Attr> {
        let mut table_list = Vec::new();
        for (name, t) in self.tables.iter() {
            if let Some(attr) = t.attr_list.iter().filter(|a| a.name == attr).next() {
                if let &Some(ref table_name) = table {
                    if *table_name == name.to_string() {
                        table_list.push(attr.clone());
                    }
                } else {
                    table_list.push(attr.clone());
                }
            }
        }
        match table_list.len() {
            1 => table_list.pop(),
            _ => None,  // not found or multiple attribute found
        }
    }
    pub fn gen_attr_list(&self, table : &String) -> Vec<Attr> {
        // table should exist
        self.tables.get(table).unwrap().attr_list.clone()
    }
    pub fn add_table(&mut self, table : Table) {
        self.tables.insert(table.name.clone(), table);
    }
}


pub type TableManagerRef = Rc<RefCell<TableManager>>;

#[derive(Debug)]
pub struct TableManager {
    tables : BTreeMap<String, TableRef>,
}

impl TableManager {
    pub fn make_ref() -> TableManagerRef {
        Rc::new(RefCell::new(TableManager::new()))
    }
    pub fn new() -> TableManager {
        TableManager{ tables : BTreeMap::new() }
    }
    pub fn save_to_file() {}
    pub fn create_table() {}
    pub fn from_json(json : &str) -> TableManager {
        let mut tables = BTreeMap::new();
        let tree : BTreeMap<String, Table> = unwrap!(decode(json));
        for (name, table) in tree.iter() {
            tables.insert(name.clone(), Rc::new(RefCell::new(table.clone())));
        }
        TableManager{
            tables : tables
        }
    }
    pub fn to_json(&self) -> String {
        let mut tree : BTreeMap<String, Table> = BTreeMap::new();
        for (name, table) in self.tables.iter() {
            tree.insert(name.clone(), table.borrow().clone());
        }
        unwrap!(encode(&tree))
    }
    pub fn add_table(&mut self, table : Table) {
        self.tables.insert(table.name.clone(), Rc::new(RefCell::new(table)));
    }
    pub fn remove_table(&mut self, table : &String) {
        self.tables.remove(table);
    }
    pub fn get_table(&self, name : &str) -> Option<TableRef> {
        match self.tables.get(name) {
            Some(ref mut table) => Some(table.clone()),
            None => None,
        }
    }
    pub fn gen_table_set(&self, used_table : &Vec<String>) -> TableSet {
        let mut tables = HashMap::new();
        for name in used_table.iter() {
            tables.insert(name.clone(), self.tables.get(name).unwrap().borrow().clone());
        }
        TableSet{ tables : tables }
    }
}

impl Encodable for AttrType {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        let mut t = BTreeMap::new();
        match self {
            &AttrType::Int => t.insert("type".to_string(), "Int".to_string()),
            &AttrType::Float => t.insert("type".to_string(), "Float".to_string()),
            &AttrType::Char{len} => {
                t.insert("type".to_string(), "Char".to_string());
                t.insert("len".to_string(), len.to_string())
            }
        };
        t.encode(s)
    }
}

impl Decodable for AttrType {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        let t : BTreeMap<String, String> = try!(BTreeMap::decode(d));
        let res = match t.get("type") {
            None => panic!("can't find key 'type' in AttrType json data"),
            Some(ref s) => match &s[..] {
                "Int" => AttrType::Int,
                "Float" => AttrType::Float,
                "Char" => {
                    let len = match t.get("len") {
                        None => panic!("can't find key 'len' for Char in AttrType json data"),
                        Some(len) => len.parse::<usize>().unwrap(),
                    };
                    AttrType::Char{ len : len }
                }
                _ => panic!("unexpected type {}", s),
            }
        };
        Ok(res)
    }
}
