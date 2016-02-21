use ::store::table::{TableManager, Table, Attr, AttrType};
use ::parser::compile_error::CompileErrorType;
use ::parser::select::SelectStatement;
use ::parser::update::UpdateStatement;
use ::parser::insert::InsertStatement;
use ::parser::delete::DeleteStatement;
use ::parser::create_drop::{CreateStatement, DropStatement};
use ::parser::sem_check::{
    check_drop,
    check_create,
};


macro_rules! gen_result {
    ($class:ident :: $parse_func:ident, $input_str:expr) => ({
        let tokens = gen_token!($input_str);
        extract!($class::$parse_func(&mut tokens.iter()), Ok(result), result)
    })
}

macro_rules! assert_ok {
    ($check_result:expr) => (assert_pattern!($check_result, Ok(..)))
}

macro_rules! assert_err {
    ($check_result:expr, $expected_type:expr) => ({
        let error_list = extract!($check_result, Err(error_list), error_list);
        let error_type = error_list.first().unwrap().error_type;
        assert_eq!(error_type, $expected_type);
    })
}

fn add_table(manager : &mut TableManager) {
    let table = Table{
        name : "author".to_string(),
        attr_list : vec![
            Attr{
                name : "id".to_string(),
                attr_type : AttrType::Int,
                primary : true,
                nullable : false,
            },
            Attr{
                name : "name".to_string(),
                attr_type : AttrType::Char{ len : 10 },
                primary : false,
                nullable : false,
            }
        ],
    };
    manager.add_table(table);
}

#[test]
fn test_check_drop() {
    let drop_stmt = gen_result!(DropStatement::parse, "drop table author");
    let mut manager = TableManager::new();
    assert_err!(check_drop(&drop_stmt, &manager), CompileErrorType::SemTableNotExist);
    add_table(&mut manager);
    assert_ok!(check_drop(&drop_stmt, &manager));
}

#[test]
fn test_check_create() {
    {// table exist
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int not null primary)");
        let mut manager = TableManager::new();
        assert_ok!(check_create(&create_stmt, &manager));
        add_table(&mut manager);
        assert_err!(check_create(&create_stmt, &manager), CompileErrorType::SemTableExist);
    }
    {// unique primary
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int not null primary)");
        let manager = TableManager::new();
        assert_ok!(check_create(&create_stmt, &manager));
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int not null primary, num int not null primary)");
        assert_err!(check_create(&create_stmt, &manager), CompileErrorType::SemMultiplePrimary);
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int)");
        assert_err!(check_create(&create_stmt, &manager), CompileErrorType::SemNoPrimary);
    }
    {// primary not null
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int not null primary)");
        let manager = TableManager::new();
        assert_ok!(check_create(&create_stmt, &manager));
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int primary)");
        assert_err!(check_create(&create_stmt, &manager), CompileErrorType::SemNullablePrimary);
    }
    {// unique attribute
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int not null primary)");
        let manager = TableManager::new();
        assert_ok!(check_create(&create_stmt, &manager));
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int not null primary, id char(10))");
        assert_err!(check_create(&create_stmt, &manager), CompileErrorType::SemDuplicateAttr);
    }
}
