use ::parser::common::Statement;
use ::store::table::{TableManager, Table, Attr, AttrType};
use ::utils::config::Config;
use ::exec::gen_plan::gen_plan;


#[test]
fn test_create_table() {
    let config = Config::new(&r#"
        max_memory_pool_page_num = 2
        table_meta_dir = "test_file/table_meta/"
        table_file_dir = "test_file/table_file""#.to_string());
    let manager = TableManager::make_ref(&config);
    assert_pattern!(manager.borrow().get_table("msg"), None);
    let mut plan = gen_plan_helper!(
        "create table msg(id int not null primary, content char(233))", &manager);
    plan.open();
    assert_pattern!(plan.get_next(), None);
    let table = extract!(manager.borrow().get_table("msg"), Some(tab), tab);
    let tab = table.borrow();
    assert_eq!(tab.name, "msg");
    assert_eq!(tab.attr_list.len(), 2);
}

#[test]
fn test_drop_table() {
    let config = Config::new(&r#"
        max_memory_pool_page_num = 2
        table_meta_dir = "test_file/table_meta/"
        table_file_dir = "test_file/table_file""#.to_string());
    let manager = TableManager::make_ref(&config);
    let table = Table{
        name : "msg".to_string(),
        attr_list : vec![Attr{
                name : "id".to_string(),
                attr_type : AttrType::Int,
                primary : true,
                nullable : false,
            }],
    };
    manager.borrow_mut().add_table(table);
    let mut plan = gen_plan_helper!("drop table msg", &manager);
    assert_pattern!(manager.borrow().get_table("msg"), Some(..));
    plan.open();
    assert_pattern!(plan.get_next(), None);
    assert_pattern!(manager.borrow().get_table("msg"), None);
}
