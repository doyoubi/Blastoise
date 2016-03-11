use std::ptr::read;
use std::collections::HashMap;
use ::exec::change::{Insert, Delete, Update};
use ::exec::query::{FileScan, Filter};
use ::exec::error::ExecErrorType;
use ::store::tuple::TupleValue;
use ::store::table::{TableManager, Table, Attr, AttrType};
use ::utils::config::Config;
use ::utils::pointer::read_string;
use ::parser::condition::ConditionExpr;
use super::test_query::{gen_test_manager, gen_test_table};


#[test]
fn test_insert() {
    let config = Config::new(&r#"
        max_memory_pool_page_num = 2
        table_meta_dir = "test_file/table_meta/"
        table_file_dir = "test_file/table_file""#.to_string());
    let manager = TableManager::make_ref(&config);
    let table_name = "test_insert_message".to_string();
    manager.borrow_mut().add_table(gen_test_table(&table_name));
    assert_pattern!(manager.borrow().get_table(&table_name), Some(..));

    let file = manager.borrow_mut().file_manager.get_file(&table_name);
    assert_eq!(file.borrow().loaded_pages.len(), 0);

    let mut plan = gen_plan_helper!(
        "insert test_insert_message values(233, 2.3333, \"i am doyoubi\")", &manager);
    plan.open();
    assert_pattern!(plan.get_next(), None);
    assert_pattern!(plan.get_error(), None);

    assert_pattern!(manager.borrow_mut().get_tuple_value(&table_name, 0, 0), TupleValue::Int(233));
    assert_pattern!(manager.borrow_mut().get_tuple_value(&table_name, 0, 1), TupleValue::Float(2.3333));
    assert_eq!(extract!(
        manager.borrow_mut().get_tuple_value(&table_name, 0, 2), TupleValue::Char(s), s), "i am doyoubi");
}

#[test]
fn test_duplicate_primary_key() {
    let table_name = "test_change_message".to_string();
    let manager = gen_test_manager(&table_name);

    let mut scan = FileScan::new(&table_name, &manager);
    scan.open();
    assert_pattern!(scan.get_next(), Some(..));
    assert_pattern!(scan.get_next(), Some(..));
    assert_pattern!(scan.get_next(), Some(..));
    assert_pattern!(scan.get_next(), None);

    let mut plan = gen_plan_helper!(
        "insert test_change_message values(233, 2.3333, \"i am doyoubi\")", &manager);
    plan.open();
    assert_pattern!(plan.get_next(), None);
    let err = plan.get_error().unwrap();
    assert_eq!(err.error_type, ExecErrorType::PrimaryKeyExist);

    let mut scan = FileScan::new(&table_name, &manager);
    scan.open();
    assert_pattern!(scan.get_next(), Some(..));
    assert_pattern!(scan.get_next(), Some(..));
    assert_pattern!(scan.get_next(), Some(..));
    assert_pattern!(scan.get_next(), None);
}

#[test]
fn test_insert_without_creating_new_page() {
    let table_name = "test_change_message".to_string();
    let manager = gen_test_manager(&table_name);

    let mut scan = FileScan::new(&table_name, &manager);
    scan.open();
    assert_pattern!(scan.get_next(), Some(..));
    assert_pattern!(scan.get_next(), Some(..));
    assert_pattern!(scan.get_next(), Some(..));
    assert_pattern!(scan.get_next(), None);

    let mut plan = gen_plan_helper!(
        "insert test_change_message values(1234, 2.3333, \"i am doyoubi\")", &manager);
    plan.open();
    assert_pattern!(plan.get_next(), None);
    assert_pattern!(plan.get_error(), None);

    let mut scan = FileScan::new(&table_name, &manager);
    scan.open();
    assert_pattern!(scan.get_next(), Some(..));
    assert_pattern!(scan.get_next(), Some(..));
    let tuple_data = extract!(scan.get_next(), Some(tuple_data), tuple_data);
    assert_pattern!(scan.get_next(), Some(..));
    assert_pattern!(scan.get_next(), None);

    assert_eq!(unsafe{read::<i32>(tuple_data[0] as *const i32)}, 1234);
    assert_eq!(unsafe{read::<f32>(tuple_data[1] as *const f32)}, 2.3333);
    assert_eq!(unsafe{read_string(tuple_data[2], 16)}, "i am doyoubi");
}

#[test]
fn test_delete() {
    {
        // delete all
        let table_name = "test_change_message".to_string();
        let manager = gen_test_manager(&table_name);
        let mut scan = FileScan::new(&table_name, &manager);
        scan.open();
        assert_pattern!(scan.get_next(), Some(..));
        assert_pattern!(scan.get_next(), Some(..));
        assert_pattern!(scan.get_next(), Some(..));
        assert_pattern!(scan.get_next(), None);
        let mut delete = Delete::new(&table_name,
            FileScan::new(&table_name, &manager), &manager);
        delete.open();
        assert_pattern!(delete.get_next(), Some(..));
        assert_pattern!(delete.get_next(), Some(..));
        assert_pattern!(delete.get_next(), Some(..));
        assert_pattern!(delete.get_next(), None);
        let mut new_scan = FileScan::new(&table_name, &manager);
        new_scan.open();
        assert_pattern!(new_scan.get_next(), None);
    }
    {
        // delete with where clause
        let table_name = "test_change_message".to_string();
        let manager = gen_test_manager(&table_name);
        let mut scan = FileScan::new(&table_name, &manager);
        scan.open();
        assert_pattern!(scan.get_next(), Some(..));
        assert_pattern!(scan.get_next(), Some(..));
        assert_pattern!(scan.get_next(), Some(..));
        assert_pattern!(scan.get_next(), None);

        let table = gen_test_table(&table_name);
        let mut data_souce = FileScan::new(&table_name, &manager);
        let cond = Box::new(gen_parse_result!(ConditionExpr::parse,
            "test_change_message.id = 777"));
        data_souce = Filter::new(cond, table.gen_index_map(), table.gen_tuple_desc(), data_souce);
        let mut delete = Delete::new(&table_name, data_souce, &manager);
        delete.open();
        let deleted_tuple = extract!(delete.get_next(), Some(tuple_data), tuple_data);
        assert_eq!(unsafe{ read::<i32>(deleted_tuple[0] as *const i32) }, 777);
        assert_pattern!(delete.get_next(), None);

        let mut scan = FileScan::new(&table_name, &manager);
        scan.open();
        let t1 = extract!(scan.get_next(), Some(tuple_data), tuple_data);
        let t2 = extract!(scan.get_next(), Some(tuple_data), tuple_data);
        assert_pattern!(scan.get_next(), None);
        assert_eq!(unsafe{ read::<i32>(t1[0] as *const i32) }, 233);
        assert_eq!(unsafe{ read::<i32>(t2[0] as *const i32) }, 1);
    }
}

#[test]
fn test_update() {
    {
        // update all
        let table_name = "test_change_message".to_string();
        let manager = gen_test_manager(&table_name);
        let table = gen_test_table(&table_name);
        let mut set_values = HashMap::new();
        set_values.insert(1, TupleValue::Float(233.666));
        let mut update = Update::new(&table_name, table.gen_tuple_desc(), set_values,
            FileScan::new(&table_name, &manager), &manager);
        update.open();
        assert_pattern!(update.get_next(), Some(..));
        assert_pattern!(update.get_next(), Some(..));
        assert_pattern!(update.get_next(), Some(..));
        assert_pattern!(update.get_next(), None);
        let mut scan = FileScan::new(&table_name, &manager);
        scan.open();
        let t1 = extract!(scan.get_next(), Some(tuple_data), tuple_data);
        let t2 = extract!(scan.get_next(), Some(tuple_data), tuple_data);
        let t3 = extract!(scan.get_next(), Some(tuple_data), tuple_data);
        assert_pattern!(scan.get_next(), None);
        assert_eq!(unsafe{ read::<f32>(t1[1] as *const f32) }, 233.666);
        assert_eq!(unsafe{ read::<f32>(t2[1] as *const f32) }, 233.666);
        assert_eq!(unsafe{ read::<f32>(t3[1] as *const f32) }, 233.666);
    }
    {
        // update with where clause
        let table_name = "test_change_message".to_string();
        let manager = gen_test_manager(&table_name);
        let table = gen_test_table(&table_name);

        let mut scan = FileScan::new(&table_name, &manager);
        scan.open();
        let t1 = extract!(scan.get_next(), Some(tuple_data), tuple_data);
        let t2 = extract!(scan.get_next(), Some(tuple_data), tuple_data);
        let t3 = extract!(scan.get_next(), Some(tuple_data), tuple_data);
        assert_pattern!(scan.get_next(), None);
        assert_eq!(unsafe{ read::<f32>(t1[1] as *const f32) }, 666.666);
        assert_eq!(unsafe{ read::<f32>(t2[1] as *const f32) }, 12345.777);
        assert_eq!(unsafe{ read::<f32>(t3[1] as *const f32) }, 123.0);

        let mut set_values = HashMap::new();
        set_values.insert(1, TupleValue::Float(233.666));
        let mut data_souce = FileScan::new(&table_name, &manager);
        let cond = Box::new(gen_parse_result!(ConditionExpr::parse,
            "test_change_message.id = 777"));
        data_souce = Filter::new(cond, table.gen_index_map(), table.gen_tuple_desc(), data_souce);
        let mut update = Update::new(&table_name, table.gen_tuple_desc(), set_values,
            data_souce, &manager);
        update.open();
        let updated_tuple = extract!(update.get_next(), Some(tuple_data), tuple_data);
        assert_eq!(unsafe{ read::<i32>(updated_tuple[0] as *const i32) }, 777);
        assert_eq!(unsafe{ read::<f32>(updated_tuple[1] as *const f32) }, 233.666);
        assert_pattern!(update.get_next(), None);

        let mut scan = FileScan::new(&table_name, &manager);
        scan.open();
        let t1 = extract!(scan.get_next(), Some(tuple_data), tuple_data);
        let t2 = extract!(scan.get_next(), Some(tuple_data), tuple_data);
        let t3 = extract!(scan.get_next(), Some(tuple_data), tuple_data);
        assert_pattern!(scan.get_next(), None);
        assert_eq!(unsafe{ read::<f32>(t1[1] as *const f32) }, 666.666);
        assert_eq!(unsafe{ read::<f32>(t2[1] as *const f32) }, 233.666);
        assert_eq!(unsafe{ read::<f32>(t3[1] as *const f32) }, 123.0);
    }
}

