use ::store::table::{TableSet, Table, Attr, AttrType};
use ::parser::condition::ConditionExpr;
use ::parser::compile_error::CompileErrorType;
use ::parser::select::SelectStatement;
use ::parser::update::UpdateStatement;
use ::parser::insert::InsertStatement;
use ::parser::delete::DeleteStatement;
use ::parser::create_drop::{CreateStatement, DropStatement};
use ::parser::sem_check::{
    check_drop,
    check_create,
    check_condition,
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

fn add_table(table_set : &mut TableSet) {
    let t1 = Table{
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
    let t2 = Table{
        name : "book".to_string(),
        attr_list : vec![
            Attr{
                name : "id".to_string(),
                attr_type : AttrType::Int,
                primary : true,
                nullable : false,
            },
            Attr{
                name : "author_id".to_string(),
                attr_type : AttrType::Int,
                primary : true,
                nullable : true,
            }
        ]
    };
    table_set.add_table(t1);
    table_set.add_table(t2);
}

#[test]
fn test_check_drop() {
    let drop_stmt = gen_result!(DropStatement::parse, "drop table author");
    let mut table_set = TableSet::new();
    assert_err!(check_drop(&drop_stmt, &table_set), CompileErrorType::SemTableNotExist);
    add_table(&mut table_set);
    assert_ok!(check_drop(&drop_stmt, &table_set));
}

#[test]
fn test_check_create() {
    {// table exist
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int not null primary)");
        let mut table_set = TableSet::new();
        assert_ok!(check_create(&create_stmt, &table_set));
        add_table(&mut table_set);
        assert_err!(check_create(&create_stmt, &table_set), CompileErrorType::SemTableExist);
    }
    {// unique primary
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int not null primary)");
        let table_set = TableSet::new();
        assert_ok!(check_create(&create_stmt, &table_set));
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int not null primary, num int not null primary)");
        assert_err!(check_create(&create_stmt, &table_set), CompileErrorType::SemMultiplePrimary);
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int)");
        assert_err!(check_create(&create_stmt, &table_set), CompileErrorType::SemNoPrimary);
    }
    {// primary not null
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int not null primary)");
        let table_set = TableSet::new();
        assert_ok!(check_create(&create_stmt, &table_set));
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int primary)");
        assert_err!(check_create(&create_stmt, &table_set), CompileErrorType::SemNullablePrimary);
    }
    {// unique attribute
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int not null primary)");
        let table_set = TableSet::new();
        assert_ok!(check_create(&create_stmt, &table_set));
        let create_stmt = gen_result!(CreateStatement::parse,
            "create table author(id int not null primary, id char(10))");
        assert_err!(check_create(&create_stmt, &table_set), CompileErrorType::SemDuplicateAttr);
    }
}

#[test]
fn test_check_condition() {
    // arithmatic type correctness already guranteed by grammar
    {// comparasion type check
        let mut table_set = TableSet::new();
        let condition = gen_result!(ConditionExpr::parse, "1 < 0 or 1 = 2");
        assert_ok!(check_condition(&condition, &table_set, &None));

        let condition = gen_result!(ConditionExpr::parse, "1 = null");
        assert_err!(check_condition(&condition, &table_set, &None), CompileErrorType::SemInvalidValueType);

        let condition = gen_result!(ConditionExpr::parse, "1 < \"i am string\"");
        assert_err!(check_condition(&condition, &table_set, &None), CompileErrorType::SemInvalidValueType);
        
        let condition = gen_result!(ConditionExpr::parse, "\"aaa\" = \"bbb\"");
        assert_ok!(check_condition(&condition, &table_set, &None));

        add_table(&mut table_set);
        let condition = gen_result!(ConditionExpr::parse, "author_id is not null");
        assert_ok!(check_condition(&condition, &table_set, &None));

        let condition = gen_result!(ConditionExpr::parse, "author_id is 1");
        assert_err!(check_condition(&condition, &table_set, &None), CompileErrorType::SemInvalidValueType);

        let condition = gen_result!(ConditionExpr::parse, "2 is null");
        assert_err!(check_condition(&condition, &table_set, &None), CompileErrorType::SemInvalidValueType);
    }
    {// attirbute check
        let mut table_set = TableSet::new();
        let condition = gen_result!(ConditionExpr::parse, "a is null");
        assert_err!(check_condition(&condition, &table_set, &None), CompileErrorType::SemInvalidAttribute);

        add_table(&mut table_set);
        let condition = gen_result!(ConditionExpr::parse, "author_id is null");
        assert_ok!(check_condition(&condition, &table_set, &None));

        let condition = gen_result!(ConditionExpr::parse, "book.author_id is null");
        assert_ok!(check_condition(&condition, &table_set, &None));

        let condition = gen_result!(ConditionExpr::parse, "book.id > 1");
        assert_ok!(check_condition(&condition, &table_set,
            &Some((Some("book".to_string()), "id".to_string()))));
        assert_err!(check_condition(&condition, &table_set,
            &Some((Some("book".to_string()), "author_id".to_string()))),
            CompileErrorType::SemShouldUseGroupByAttribute);

        let condition = gen_result!(ConditionExpr::parse, "sum(book.id) > 1");
        assert_ok!(check_condition(&condition, &table_set,
            &Some((Some("book".to_string()), "id".to_string()))));

        let condition = gen_result!(ConditionExpr::parse, "invalid_func(book.id) > 1");
        assert_err!(check_condition(&condition, &table_set,
            &Some((Some("book".to_string()), "id".to_string()))),
            CompileErrorType::SemInvalidAggreFuncName);

        let condition = gen_result!(ConditionExpr::parse, "sum(book.id) > 1");
        assert_err!(check_condition(&condition, &table_set, &None),
            CompileErrorType::SemInvalidAggregateFunctionUse);

        let condition = gen_result!(ConditionExpr::parse, "name > 0");
        assert_err!(check_condition(&condition, &table_set, &None), CompileErrorType::SemInvalidValueType);

        let condition = gen_result!(ConditionExpr::parse, "name is null");
        assert_err!(check_condition(&condition, &table_set, &None),
            CompileErrorType::SemAttributeNotNullable);

        let condition = gen_result!(ConditionExpr::parse, "id is null");
        assert_err!(check_condition(&condition, &table_set, &None),
            CompileErrorType::SemInvalidAttribute);
    }
}
