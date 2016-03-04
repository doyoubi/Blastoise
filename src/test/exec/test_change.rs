use ::exec::change::Insert;
use ::store::tuple::TupleValue;
use ::store::table::{TableManager, Table, Attr, AttrType};
use ::utils::config::Config;


fn gen_test_table() -> Table {
    Table{
        name : "test_change_message".to_string(),
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

#[test]
fn test_insert() {
    let config = Config::new(&r#"
        max_memory_pool_page_num = 2
        table_file_dir = "table_file""#.to_string());
    let manager = TableManager::make_ref(&config);
    let table_name = "test_change_message".to_string();
    manager.borrow_mut().add_table(gen_test_table());
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
