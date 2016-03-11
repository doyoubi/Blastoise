use std::ptr::{read, write};
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, RwLock};
use libc::malloc;
use ::utils::pointer::{read_string, write_string, pointer_offset};
use ::utils::config::Config;
use ::store::file::{TableFile, FilePage, BitMap, PageHeader, TableFileManager};
use ::store::buffer::{DataPtr, Page};
use ::store::table::{Table, Attr, AttrType, TableManager};
use ::parser::common::{ValueExpr, ValueType};
use ::store::tuple::TupleValue;
use ::test::exec::test_query;
use ::test::exec::test_query::gen_test_manager;


#[test]
fn test_page_header() {
    {
        let data;
        unsafe{
            data = malloc(8);
            write::<u32>(data as *mut u32, 233);
            write::<u32>((data as *mut u32).offset(1), 666);
        }
        let mut header = PageHeader{
            slot_sum : 233,
            first_free_slot : 0,
            data : data,
        };
        header.init_from_page_data();
        assert_eq!(header.slot_sum, 233);
        assert_eq!(header.first_free_slot, 666);
    }
    {
        let data;
        unsafe{
            data = malloc(8);
            write::<u32>(data as *mut u32, 111);
            write::<u32>((data as *mut u32).offset(1), 222);
        }
        let mut header = PageHeader{
            slot_sum : 233,
            first_free_slot : 666,
            data : data,
        };
        header.save_to_page_data();
        assert_eq!(unsafe{read::<u32>(data as *const u32)}, 233);
        assert_eq!(unsafe{read::<u32>((data as *const u32).offset(1))}, 666);
    }
}

#[test]
fn test_bitmap()
{
    let data = unsafe{ malloc(3) };
    let mut bitmap = BitMap{
        data : data,
        slot_sum : 24,
    };
    bitmap.clean();
    for i in 0..24 {
        assert!(!bitmap.is_inuse(i));
    }
    bitmap.set_inuse(8, true);
    assert!(!bitmap.is_inuse(7));
    assert!(bitmap.is_inuse(8));
    assert!(!bitmap.is_inuse(9));
    assert_eq!(bitmap.next_tuple_index(0), 8);
    bitmap.set_inuse(8 + 7, true);
    assert!(!bitmap.is_inuse(8 + 6));
    assert!(bitmap.is_inuse(8 + 7));
    assert!(!bitmap.is_inuse(8 + 8));
    assert_eq!(bitmap.next_tuple_index(0), 8);
    assert_eq!(bitmap.next_tuple_index(8), 8);
    assert_eq!(bitmap.next_tuple_index(8 + 1), 8 + 7);
    bitmap.set_inuse(8 + 3, true);
    assert!(!bitmap.is_inuse(8 + 2));
    assert!(bitmap.is_inuse(8 + 3));
    assert!(!bitmap.is_inuse(8 + 4));
    unsafe{
        assert_eq!(read::<u8>(data as *const u8), 0);
        assert_eq!(read::<u8>((data as *const u8).offset(1)), 137);
        assert_eq!(read::<u8>((data as *const u8).offset(2)), 0);
    }
    assert_eq!(bitmap.get_first_free_slot(), 0);
    bitmap.set_inuse(0, true);
    assert_eq!(bitmap.get_first_free_slot(), 1);
    for i in 1..8 {
        bitmap.set_inuse(i, true);
    }
    assert_eq!(bitmap.next_tuple_index(8), 8);
    assert_eq!(bitmap.get_first_free_slot(), 8 + 1);
    bitmap.set_inuse(3, false);
    assert_eq!(bitmap.get_first_free_slot(), 3);
    assert_eq!(bitmap.next_tuple_index(50), bitmap.slot_sum);
}

fn gen_test_table() -> Table {
    Table{
        name : "test_file_message".to_string(),
        attr_list : vec![
            Attr{
                name : "id".to_string(),
                attr_type : AttrType::Int,
                primary : true,
                nullable : false,
            },
            Attr{
                name : "content".to_string(),
                attr_type : AttrType::Char{ len : 6 },
                primary : false,
                nullable : false,
            },
            Attr{
                name : "score".to_string(),
                attr_type : AttrType::Float,
                primary : false,
                nullable : true,
            },
        ],
    }
}

#[test]
fn test_file_page_insert() {
    let table = gen_test_table();
    let tuple_desc = table.gen_tuple_desc();
    assert_eq!(tuple_desc.tuple_len, 16);
    let mut mem_page = Page::new(1, 2);
    mem_page.alloc();
    let page = Rc::new(RefCell::new(mem_page));
    let mut file_page = FilePage::new(page, tuple_desc.tuple_len);
    file_page.init_empty_page();
    let mut value_list = vec![
        ValueExpr{ value : "233".to_string(), value_type : ValueType::Integer },
        ValueExpr{ value : "abcdef".to_string(), value_type : ValueType::String },
        ValueExpr{ value : "666.666".to_string(), value_type : ValueType::Float },
    ];
    assert_eq!(file_page.header.first_free_slot, 0);
    assert_eq!(file_page.is_inuse(0), false);
    file_page.insert(&value_list, &tuple_desc);
    assert_eq!(file_page.header.first_free_slot, 1);
    assert_eq!(file_page.is_inuse(0), true);
    value_list[0].value = "777".to_string();
    value_list[1].value = "dyb".to_string();
    value_list[2].value = "12345.777".to_string();
    file_page.insert(&value_list, &tuple_desc);
    assert_eq!(file_page.header.first_free_slot, 2);
    assert_eq!(file_page.is_inuse(1), true);

    file_page.save_to_page();
    let mut p = file_page.mem_page.borrow().data;
    assert_eq!(unsafe{ read(p as *const u32) }, 253);  // slot_sum
    p = pointer_offset(p, 4);
    assert_eq!(unsafe{ read(p as *const u32) }, 2);  // first_free_slot
    p = pointer_offset(p, 4);
    assert_eq!(unsafe{ read(p as *const u8) }, 3);  // bitmap
    assert_eq!(unsafe{ read(pointer_offset(p, 4) as *const u8) }, 0);  // bitmap
    p = pointer_offset(p, (253 + 7) / 8);
    // first tuple
    assert_eq!(unsafe{ read(p as *const u32) }, 233);  // tuple data: id
    p = pointer_offset(p, 4);
    assert_eq!(unsafe{ read_string(p, 6) }, "abcdef");  // tuple data: content
    p = pointer_offset(p, 8);
    assert_eq!(unsafe{ read(p as *const f32) }, 666.666);  // tuple data: score
    p = pointer_offset(p, 4);
    // second tuple
    assert_eq!(unsafe{ read(p as *const u32) }, 777);  // tuple data: id
    p = pointer_offset(p, 4);
    assert_eq!(unsafe{ read_string(p, 6) }, "dyb");  // tuple data: content
    p = pointer_offset(p, 8);
    assert_eq!(unsafe{ read(p as *const f32) }, 12345.777);  // tuple data: score
}

#[test]
fn test_file_insert() {
    let config = Config::new(&r#"
        max_memory_pool_page_num = 2
        table_meta_dir = "test_file/table_meta/"
        table_file_dir = "test_file/table_file""#.to_string());
    let mut manager = TableFileManager::new(&config);
    let table = Rc::new(RefCell::new(gen_test_table()));
    let table_name = "test_file_message".to_string();
    manager.create_file(table_name.clone(), table);
    let mut value_list = vec![
        ValueExpr{ value : "233".to_string(), value_type : ValueType::Integer },
        ValueExpr{ value : "abcdef".to_string(), value_type : ValueType::String },
        ValueExpr{ value : "666.666".to_string(), value_type : ValueType::Float },
    ];
    manager.insert(&table_name, &value_list);
    assert_pattern!(manager.get_tuple_value(&table_name, 0, 0), TupleValue::Int(233));
    assert_pattern!(manager.get_tuple_value(&table_name, 0, 2), TupleValue::Float(666.666));
    assert_eq!(extract!(
        manager.get_tuple_value(&table_name, 0, 1), TupleValue::Char(s), s), "abcdef");

    value_list[0].value = "777".to_string();
    value_list[1].value = "dyb".to_string();
    value_list[2].value = "12345.777".to_string();
    manager.insert(&table_name, &value_list);
    assert_pattern!(manager.get_tuple_value(&table_name, 1, 0), TupleValue::Int(777));
    assert_pattern!(manager.get_tuple_value(&table_name, 1, 2), TupleValue::Float(12345.777));
    assert_eq!(extract!(
        manager.get_tuple_value(&table_name, 1, 1), TupleValue::Char(s), s), "dyb");
}

#[test]
fn test_get_tuple_data() {
    let config = Config::new(&r#"
        max_memory_pool_page_num = 2
        table_meta_dir = "test_file/table_meta/"
        table_file_dir = "test_file/table_file""#.to_string());
    let mut manager = TableFileManager::new(&config);
    let table = Rc::new(RefCell::new(gen_test_table()));
    let table_name = "test_file_message".to_string();
    manager.create_file(table_name.clone(), table);
    let value_list = vec![
        ValueExpr{ value : "233".to_string(), value_type : ValueType::Integer },
        ValueExpr{ value : "abcdef".to_string(), value_type : ValueType::String },
        ValueExpr{ value : "666.666".to_string(), value_type : ValueType::Float },
    ];
    manager.insert(&table_name, &value_list);
    let tuple_data = manager.get_tuple_data(&table_name, 0).unwrap();
    let p1 = tuple_data[0];
    let p2 = tuple_data[1];
    let p3 = tuple_data[2];
    assert_eq!(unsafe{ read::<i32>(p1 as *const i32) }, 233);
    assert_eq!(unsafe{ read_string(p2, 6) }, "abcdef");
    assert_eq!(unsafe{ read::<f32>(p3 as *const f32) }, 666.666);
}

#[test]
fn test_file_persistence() {
    {
        let config = Config::new(&r#"
            max_memory_pool_page_num = 2
            table_meta_dir = "test_file/table_meta/test_file_persistence/"
            table_file_dir = "test_file/table_file""#.to_string());
        let table_name = "test_file_persistence_message".to_string();
        {
            let manager = Rc::new(RefCell::new(TableManager::new(&config)));
            manager.borrow_mut().add_table(test_query::gen_test_table(&table_name));
            test_query::insert_data(&table_name, &manager);
            manager.borrow_mut().save_to_file();
        }
        let manager = Rc::new(RefCell::new(TableManager::from_json_file(&config)));
        let file = manager.borrow_mut().file_manager.get_file(&table_name);
        assert_eq!(file.borrow().page_sum, 2);
        assert_eq!(file.borrow().first_free_page, 0);
        let mut query = gen_plan_helper!(
            "select * from test_file_persistence_message", &manager);
        query.open();
        let t1 = extract!(query.get_next(), Some(tuple_data), tuple_data);
        let t2 = extract!(query.get_next(), Some(tuple_data), tuple_data);
        let t3 = extract!(query.get_next(), Some(tuple_data), tuple_data);
        assert_pattern!(query.get_next(), None);
        assert_eq!(unsafe{ read::<i32>(t1[0] as *const i32) }, 233);
        assert_eq!(unsafe{ read::<i32>(t2[0] as *const i32) }, 777);
        assert_eq!(unsafe{ read::<i32>(t3[0] as *const i32) }, 1);
    }
    {
        // test page switch
        let config = Config::new(&r#"
            max_memory_pool_page_num = 2
            table_meta_dir = "test_file/table_meta/test_persistence_with_page_switch/"
            table_file_dir = "test_file/table_file/test_persistence_with_page_switch/""#.to_string());
        let table_name = "test_file_persistence_message".to_string();
        let add_table_name = "test_file_message".to_string();
        {
            let manager = Rc::new(RefCell::new(TableManager::new(&config)));
            manager.borrow_mut().add_table(test_query::gen_test_table(&table_name));
            test_query::insert_data(&table_name, &manager);

            assert_eq!(manager.borrow().file_manager.page_pool.get_capacity(), 2);
            manager.borrow_mut().add_table(gen_test_table());

            let mut insert = gen_plan_helper!(
                "insert test_file_message values(766, \"test\", 2.3333)",
                &manager);
            insert.open();
            assert_pattern!(insert.get_next(), None);
            assert_pattern!(insert.get_error(), None);

            let mut scan = gen_plan_helper!(
                "select * from test_file_message", &manager);
            scan.open();
            assert_pattern!(scan.get_next(), Some(..));
            assert_pattern!(insert.get_next(), None);

            manager.borrow_mut().save_to_file();
        }
        let manager = Rc::new(RefCell::new(TableManager::from_json_file(&config)));
        {
            let file = manager.borrow_mut().file_manager.get_file(&table_name);
            assert_eq!(file.borrow().page_sum, 2);
            assert_eq!(file.borrow().first_free_page, 0);
            let mut query = gen_plan_helper!(
                "select * from test_file_persistence_message", &manager);
            query.open();
            let t1 = extract!(query.get_next(), Some(tuple_data), tuple_data);
            let t2 = extract!(query.get_next(), Some(tuple_data), tuple_data);
            let t3 = extract!(query.get_next(), Some(tuple_data), tuple_data);
            assert_pattern!(query.get_next(), None);
            assert_eq!(unsafe{ read::<i32>(t1[0] as *const i32) }, 233);
            assert_eq!(unsafe{ read::<i32>(t2[0] as *const i32) }, 777);
            assert_eq!(unsafe{ read::<i32>(t3[0] as *const i32) }, 1);
        }
        {
            let file = manager.borrow_mut().file_manager.get_file(&add_table_name);
            assert_eq!(file.borrow().page_sum, 1);
            assert_eq!(file.borrow().first_free_page, 0);
            let mut query = gen_plan_helper!(
                "select * from test_file_message", &manager);
            query.open();
            let t1 = extract!(query.get_next(), Some(tuple_data), tuple_data);
            assert_pattern!(query.get_next(), None);
            assert_eq!(unsafe{ read::<i32>(t1[0] as *const i32) }, 766);
        }
    }
}
