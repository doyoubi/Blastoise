use std::vec::Vec;
use std::collections::BTreeMap;
use std::option::Option;
use std::sync::{RwLock, Mutex, Arc};
use rustc_serialize::{Encodable, Decodable, Encoder, Decoder};
use rustc_serialize::json::{encode, decode};


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

pub type TableRef = Arc<RwLock<Table>>;

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Table {
    pub name : String,
    pub attr_list : Vec<Attr>,
}


pub type TableManagerRef = Arc<Mutex<TableManager>>;

#[derive(Debug)]
pub struct TableManager {
    tables : BTreeMap<String, TableRef>,
}

impl TableManager {
    pub fn new() -> TableManager {
        TableManager{ tables : BTreeMap::new() }
    }
    pub fn save_to_file() {}
    pub fn create_table() {}
    pub fn from_json(json : &str) -> TableManager {
        let mut tables = BTreeMap::new();
        let tree : BTreeMap<String, Table> = unwrap!(decode(json));
        for (name, table) in tree.iter() {
            tables.insert(name.clone(), Arc::new(RwLock::new(table.clone())));
        }
        TableManager{
            tables : tables
        }
    }
    pub fn to_json(&self) -> String {
        let mut tables = Vec::new();  // hold lock
        let mut tree : BTreeMap<String, Table> = BTreeMap::new();
        for (name, table) in self.tables.iter() {
            let t = lock_unwrap!(table.read());
            tree.insert(name.clone(), t.clone());
            tables.push(t);
        }
        unwrap!(encode(&tree))
    }
    pub fn add_table(&mut self, table : Table) {
        self.tables.insert(table.name.clone(), Arc::new(RwLock::new(table)));
    }
    pub fn get_table(&self, name : &str) -> Option<TableRef> {
        match self.tables.get(name) {
            Some(table_ref) => Some(table_ref.clone()),
            None => None,
        }
    }
    pub fn exist(&self, name : &str) -> bool {
        match self.tables.get(name) {
            Some(..) => true,
            None => false,
        }
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
