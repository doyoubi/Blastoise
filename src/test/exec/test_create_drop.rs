use ::parser::common::Statement;
use ::parser::sem_check::check_sem;
use ::store::table::{TableManager, Table, Attr, AttrType};
use ::exec::gen_plan::{
    gen_table_set,
    gen_plan
};


macro_rules! gen_plan_helper {
    ($input_str:expr, $manager:expr) => ({
        let tokens = gen_token!($input_str);
        let stmt = Statement::parse(&mut tokens.iter());
        let stmt = extract!(stmt, Ok(stmt), stmt);
        {
            let lock = $manager.lock().unwrap();
            let table_set = gen_table_set(&stmt, &lock);
            assert_pattern!(check_sem(&stmt, &table_set), Ok(()));
        }
        gen_plan(stmt, $manager).unwrap()
    })
}

#[test]
fn test_create_table() {
    let manager = TableManager::make_ref();
    assert_pattern!(manager.lock().unwrap().get_table("msg"), None);
    let mut plan = gen_plan_helper!(
        "create table msg(id int not null primary, content char(233))", &manager);
    plan.open();
    assert_pattern!(plan.get_next(), None);
    let table = extract!(manager.lock().unwrap().get_table("msg"), Some(tab), tab);
    let tab = table.read().unwrap();
    assert_eq!(tab.name, "msg");
    assert_eq!(tab.attr_list.len(), 2);
}

#[test]
fn test_drop_table() {
    let manager = TableManager::make_ref();
    let table = Table{
        name : "msg".to_string(),
        attr_list : vec![Attr{
                name : "id".to_string(),
                attr_type : AttrType::Int,
                primary : true,
                nullable : false,
            }],
    };
    manager.lock().unwrap().add_table(table);
    let mut plan = gen_plan_helper!("drop table msg", &manager);
    assert_pattern!(manager.lock().unwrap().get_table("msg"), Some(..));
    plan.open();
    assert_pattern!(plan.get_next(), None);
    assert_pattern!(manager.lock().unwrap().get_table("msg"), None);
}
