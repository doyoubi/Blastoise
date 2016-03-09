use std::vec::Vec;
use std::collections::{BTreeMap, HashMap};
use std::option::Option;
use std::rc::Rc;
use std::cell::RefCell;
use rustc_serialize::{Encodable, Decodable, Encoder, Decoder};
use rustc_serialize::json::{encode, decode};
use ::parser::common::ValueList;
use ::utils::config::Config;
use ::store::tuple::TupleValue;
use super::tuple::TupleDesc;
use super::file::TableFileManager;


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
pub type IndexMap = HashMap<(String, String), usize>;

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Table {
    pub name : String,
    pub attr_list : Vec<Attr>,
}

impl Table {
    pub fn gen_tuple_desc(&self) -> TupleDesc {
        TupleDesc::new(&self.attr_list)
    }
    pub fn gen_index_map(&self) -> IndexMap {
        let mut index_map = IndexMap::new();
        for (i, attr) in self.attr_list.iter().enumerate() {
            index_map.insert((self.name.clone(), attr.name.clone()), i);
        }
        index_map
    }
    pub fn get_primary_key_attr(&self) -> Attr {
        self.attr_list.iter().filter(|a| a.primary).next().unwrap().clone()
    }
    pub fn get_primary_key_index(&self) -> usize {
        let mut index = 0;
        for attr in self.attr_list.iter() {
            if attr.primary {
                return index;
            }
            index += 1;
        }
        index
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
    pub fn complete_table_name(&self, table : &mut Option<String>, attr : &mut String) {
        // should called after get_attr to confirm only one result exist
        if table.is_some() { return; }
        for (name, t) in self.tables.iter() {
            if let Some(..) = t.attr_list.iter().filter(|a| a.name == *attr).next() {
                *table = Some(name.clone());
                return;
            }
        }
        panic!("attribute not exist");
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
    pub file_manager : TableFileManager,
}

impl TableManager {
    pub fn make_ref(config : &Config) -> TableManagerRef {
        Rc::new(RefCell::new(TableManager::new(config)))
    }
    pub fn new(config : &Config) -> TableManager {
        TableManager{
            tables : BTreeMap::new(),
            file_manager : TableFileManager::new(config),
        }
    }
    pub fn save_to_file() {}
    pub fn from_json(config : &Config, json : &str) -> TableManager {
        let mut tables = BTreeMap::new();
        let tree : BTreeMap<String, Table> = unwrap!(decode(json));
        for (name, table) in tree.iter() {
            tables.insert(name.clone(), Rc::new(RefCell::new(table.clone())));
        }
        let mut manager = Self::new(config);
        manager.tables = tables;
        manager
    }
    pub fn to_json(&self) -> String {
        let mut tree : BTreeMap<String, Table> = BTreeMap::new();
        for (name, table) in self.tables.iter() {
            tree.insert(name.clone(), table.borrow().clone());
        }
        unwrap!(encode(&tree))
    }
    pub fn add_table(&mut self, table : Table) {
        let name = table.name.clone();
        let table_ref = Rc::new(RefCell::new(table));
        self.file_manager.create_file(name.clone(), table_ref.clone());
        self.tables.insert(name, table_ref);
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
    pub fn get_tuple_value(&mut self, table : &String,
            position : usize,
            attr_position : usize) -> TupleValue{
        self.file_manager.get_tuple_value(table, position, attr_position)
    }
    pub fn insert(&mut self, table : &String, value_list : &ValueList) {
        self.file_manager.insert(table, value_list);
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
