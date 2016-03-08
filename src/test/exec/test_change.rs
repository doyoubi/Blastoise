use std::ptr::read;
use ::exec::change::{Insert, Delete};
use ::exec::query::{FileScan, Filter};
use ::store::tuple::TupleValue;
use ::store::table::{TableManager, Table, Attr, AttrType};
use ::utils::config::Config;
use ::parser::condition::ConditionExpr;
use super::test_query::{gen_test_manager, gen_test_table};


#[test]
fn test_insert() {
    let config = Config::new(&r#"
        max_memory_pool_page_num = 2
        table_file_dir = "table_file""#.to_string());
    let manager = TableManager::make_ref(&config);
    let table_name = "test_change_message".to_string();
    manager.borrow_mut().add_table(gen_test_table(&table_name));
    assert_pattern!(manager.borrow().get_table(&table_name), Some(..));
    let mut plan = gen_plan_helper!(
        "insert test_change_message values(233, 2.3333, \"i am doyoubi\")", &manager);
    plan.open();
    assert_pattern!(plan.get_next(), None);

    assert_pattern!(manager.borrow_mut().get_tuple_value(&table_name, 0, 0), TupleValue::Int(233));
    assert_pattern!(manager.borrow_mut().get_tuple_value(&table_name, 0, 1), TupleValue::Float(2.3333));
    assert_eq!(extract!(
        manager.borrow_mut().get_tuple_value(&table_name, 0, 2), TupleValue::Char(s), s), "i am doyoubi");
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
