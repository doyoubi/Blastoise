use std::vec::Vec;
use std::collections::{BTreeMap, HashMap};
use std::option::Option;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard, Mutex, Arc};
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


pub struct TableSet<'a> {
    pub tables : HashMap<String, Table>,
    table_refs : HashMap<String, TableRef>,
    read_locks : HashMap<String, RwLockReadGuard<'a, Table>>,
    write_locks : HashMap<String, RwLockWriteGuard<'a, Table>>,
}

impl<'a> TableSet<'a> {
    pub fn new() -> TableSet<'a> {
        TableSet{
            tables : HashMap::new(),
            table_refs : HashMap::new(),
            read_locks : HashMap::new(),
            write_locks : HashMap::new(),
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
    pub fn need_write(&self, name : &str) -> bool {
        assert!(self.exist(name));
        if let Some(..) = self.read_locks.get(name) {
            return false;
        } else if let Some(..) = self.write_locks.get(name) {
            return true;
        }
        panic!("invalid state");
    }
}


pub type TableManagerRef = Arc<Mutex<TableManager>>;

#[derive(Debug)]
pub struct TableManager {
    tables : BTreeMap<String, TableRef>,
}

impl TableManager {
    pub fn make_ref() -> TableManagerRef {
        TableManagerRef::new(Mutex::new(TableManager::new()))
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
    pub fn remove_table(&mut self, table : &String) {
        self.tables.remove(table);
    }
    pub fn get_table(&self, name : &str) -> Option<TableRef> {
        match self.tables.get(name) {
            Some(table_ref) => Some(table_ref.clone()),
            None => None,
        }
    }
    pub fn gen_table_set(&self, lock_table : &HashMap<String, bool>) -> TableSet {
        let mut tables = HashMap::new();
        let mut table_refs = HashMap::new();
        let mut write_locks = HashMap::new();
        let mut read_locks = HashMap::new();
        for (name, need_write) in lock_table.iter() {
            let table_ref = self.tables.get(name).unwrap();
            table_refs.insert(name.clone(), table_ref.clone());
            if *need_write {
                let guard = table_ref.write().unwrap();
                tables.insert(name.clone(), guard.clone());
                write_locks.insert(name.clone(), guard);
            } else {
                let guard = table_ref.read().unwrap();
                tables.insert(name.clone(), guard.clone());
                read_locks.insert(name.clone(), guard);
            }
        }
        TableSet{
            tables : tables,
            table_refs : table_refs,
            read_locks : read_locks,
            write_locks : write_locks,
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
