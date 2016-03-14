use std::rc::Rc;
use std::cell::RefCell;
use std::ptr::read;
use ::utils::pointer::read_string;
use ::store::table::{TableManagerRef, TableManager, Table, Attr, AttrType};
use ::store::file::TableFileManager;
use ::parser::common::{ValueExpr, ValueType};
use ::parser::condition::ConditionExpr;
use ::utils::config::Config;
use ::exec::query::{FileScan, Filter, Projection};
use ::exec::iter::ExecIterRef;
use ::exec::gen_plan::gen_proj_info;


pub fn gen_test_table(table_name : &String) -> Table {
    Table{
        name : table_name.clone(),
        attr_list : vec![
            Attr{
                name : "id".to_string(),
                attr_type : AttrType::Int,
                primary : true,
                nullable : false,
            },
            Attr{
                name : "score".to_string(),
                attr_type : AttrType::Float,
                primary : false,
                nullable : true,
            },
            Attr{
                name : "content".to_string(),
                attr_type : AttrType::Char{ len : 16 },
                primary : false,
                nullable : false,
            },
        ],
    }
}

pub fn insert_data(table_name : &String, manager : &TableManagerRef) {
    let mut value_list = vec![
        ValueExpr{ value : "233".to_string(), value_type : ValueType::Integer },
        ValueExpr{ value : "666.666".to_string(), value_type : ValueType::Float },
        ValueExpr{ value : "qweryu".to_string(), value_type : ValueType::String },
    ];
    manager.borrow_mut().insert(&table_name, &value_list);
    value_list[0].value = "777".to_string();
    value_list[1].value = "12345.777".to_string();
    value_list[2].value = "dyb".to_string();
    manager.borrow_mut().insert(&table_name, &value_list);

    value_list[0].value = "1".to_string();
    value_list[1].value = "123.0".to_string();
    value_list[2].value = "str".to_string();

    manager.borrow_mut().file_manager.insert_in_page(&table_name, 1, &value_list);

    let file = manager.borrow_mut().file_manager.get_file(&table_name);
    assert!(file.borrow().is_inuse(0, 0));
    assert!(file.borrow().is_inuse(0, 1));
    assert!(!file.borrow().is_inuse(0, 2));
    assert!(file.borrow().is_inuse(1, 0));
}

pub fn gen_test_manager(table_name : &String) -> TableManagerRef {
    let config = Config::new(&r#"
        max_memory_pool_page_num = 2
        table_meta_dir = "test_file/table_meta/"
        table_file_dir = "test_file/table_file""#.to_string());
    let manager = Rc::new(RefCell::new(TableManager::new(&config)));
    manager.borrow_mut().add_table(gen_test_table(table_name));
    insert_data(table_name, &manager);
    manager
}

macro_rules! assert_int {
    ($p:expr, $n:expr) => ({
        let i = unsafe{ read::<i32>($p as *const i32) };
        assert_eq!(i, $n);
    })
}

macro_rules! assert_float {
    ($p:expr, $n:expr) => ({
        let i = unsafe{ read::<f32>($p as *const f32) };
        assert_eq!(i, $n);
    })
}

macro_rules! assert_str {
    ($p:expr, $s:expr) => ({
        let i = unsafe{ read_string($p, 16) };
        assert_eq!(i, $s);
    })
}

#[test]
fn test_file_scan() {
    let table_name = "test_query_message".to_string();
    let manager = gen_test_manager(&table_name);
    let mut plan = FileScan::new(&table_name, &manager);
    plan.open();
    let mut t = plan.get_next().unwrap();
    assert_int!(t[0], 233);
    assert_float!(t[1], 666.666);
    assert_str!(t[2], "qweryu");
    t = plan.get_next().unwrap();
    assert_int!(t[0], 777);
    assert_float!(t[1], 12345.777);
    assert_str!(t[2], "dyb");
    t = plan.get_next().unwrap();
    assert_int!(t[0], 1);
    assert_float!(t[1], 123.0);
    assert_str!(t[2], "str");
    assert_pattern!(plan.get_next(), None);
}

fn gen_filter_plan(expr : &str) -> ExecIterRef {
    let table_name = "test_query_message".to_string();
    let manager = gen_test_manager(&table_name);
    let scan = FileScan::new(&table_name, &manager);
    let table = gen_test_table(&table_name);
    let cond = Box::new(gen_parse_result!(ConditionExpr::parse, expr));
    Filter::new(cond, table.gen_index_map(), table.gen_tuple_desc(), scan)
}

#[test]
fn test_filter() {
    {
        let mut plan = gen_filter_plan("test_query_message.id = 1");
        plan.open();
        let tuple_data = plan.get_next().unwrap();
        assert_int!(tuple_data[0], 1);
        assert_float!(tuple_data[1], 123.0);
        assert_str!(tuple_data[2], "str");
        assert_pattern!(plan.get_next(), None);
    }
    {
        let mut plan = gen_filter_plan("test_query_message.score < 1000");
        plan.open();
        let mut tuple_data = plan.get_next().unwrap();
        assert_int!(tuple_data[0], 233);
        assert_float!(tuple_data[1], 666.666);
        assert_str!(tuple_data[2], "qweryu");
        tuple_data = plan.get_next().unwrap();
        assert_int!(tuple_data[0], 1);
        assert_float!(tuple_data[1], 123.0);
        assert_str!(tuple_data[2], "str");
        assert_pattern!(plan.get_next(), None);
    }
    {
        let mut plan = gen_filter_plan("0 < 1000");
        plan.open();
        let mut tuple_data = plan.get_next().unwrap();
        assert_int!(tuple_data[0], 233);
        assert_float!(tuple_data[1], 666.666);
        assert_str!(tuple_data[2], "qweryu");
        tuple_data = plan.get_next().unwrap();
        assert_int!(tuple_data[0], 777);
        assert_float!(tuple_data[1], 12345.777);
        assert_str!(tuple_data[2], "dyb");
        tuple_data = plan.get_next().unwrap();
        assert_int!(tuple_data[0], 1);
        assert_float!(tuple_data[1], 123.0);
        assert_str!(tuple_data[2], "str");
        assert_pattern!(plan.get_next(), None);
    }
}

#[test]
fn test_projection() {
    let table_name = "test_query_message".to_string();
    let manager = gen_test_manager(&table_name);
    let scan = FileScan::new(&table_name, &manager);
    let projs = vec![
        ("test_query_message".to_string(), "id".to_string()),
        ("test_query_message".to_string(), "content".to_string()),
    ];
    let attr_index = vec![0, 2];
    let mut plan = Projection::new(attr_index, projs, scan);
    plan.open();
    let mut tuple_data = plan.get_next().unwrap();
    assert_eq!(tuple_data.len(), 2);
    assert_int!(tuple_data[0], 233);
    assert_str!(tuple_data[1], "qweryu");
    tuple_data = plan.get_next().unwrap();
    assert_eq!(tuple_data.len(), 2);
    assert_int!(tuple_data[0], 777);
    assert_str!(tuple_data[1], "dyb");
    tuple_data = plan.get_next().unwrap();
    assert_eq!(tuple_data.len(), 2);
    assert_int!(tuple_data[0], 1);
    assert_str!(tuple_data[1], "str");
    assert_pattern!(plan.get_next(), None);
}
