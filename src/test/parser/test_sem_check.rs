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
    check_insert,
    check_update,
    check_select,
};


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
                primary : false,
                nullable : true,
            },
            Attr{
                name : "name".to_string(),
                attr_type : AttrType::Char{ len : 10},
                primary : false,
                nullable : true,
            }
        ]
    };
    table_set.add_table(t1);
    table_set.add_table(t2);
}

#[test]
fn test_check_drop() {
    let drop_stmt = gen_parse_result!(DropStatement::parse, "drop table author");
    let mut table_set = TableSet::new();
    assert_err!(check_drop(&drop_stmt, &table_set), CompileErrorType::SemTableNotExist);
    add_table(&mut table_set);
    assert_ok!(check_drop(&drop_stmt, &table_set));
}

#[test]
fn test_check_create() {
    {// table exist
        let create_stmt = gen_parse_result!(CreateStatement::parse,
            "create table author(id int not null primary)");
        let mut table_set = TableSet::new();
        assert_ok!(check_create(&create_stmt, &table_set));
        add_table(&mut table_set);
        assert_err!(check_create(&create_stmt, &table_set), CompileErrorType::SemTableExist);
    }
    {// unique primary
        let create_stmt = gen_parse_result!(CreateStatement::parse,
            "create table author(id int not null primary)");
        let table_set = TableSet::new();
        assert_ok!(check_create(&create_stmt, &table_set));
        let create_stmt = gen_parse_result!(CreateStatement::parse,
            "create table author(id int not null primary, num int not null primary)");
        assert_err!(check_create(&create_stmt, &table_set), CompileErrorType::SemMultiplePrimary);
        let create_stmt = gen_parse_result!(CreateStatement::parse,
            "create table author(id int)");
        assert_err!(check_create(&create_stmt, &table_set), CompileErrorType::SemNoPrimary);
    }
    {// primary not null
        let create_stmt = gen_parse_result!(CreateStatement::parse,
            "create table author(id int not null primary)");
        let table_set = TableSet::new();
        assert_ok!(check_create(&create_stmt, &table_set));
        let create_stmt = gen_parse_result!(CreateStatement::parse,
            "create table author(id int primary)");
        assert_err!(check_create(&create_stmt, &table_set), CompileErrorType::SemNullablePrimary);
    }
    {// unique attribute
        let create_stmt = gen_parse_result!(CreateStatement::parse,
            "create table author(id int not null primary)");
        let table_set = TableSet::new();
        assert_ok!(check_create(&create_stmt, &table_set));
        let create_stmt = gen_parse_result!(CreateStatement::parse,
            "create table author(id int not null primary, id char(10))");
        assert_err!(check_create(&create_stmt, &table_set), CompileErrorType::SemDuplicateAttr);
    }
}

#[test]
fn test_check_condition() {
    // arithmatic type correctness already guranteed by grammar
    {// comparasion type check
        let mut table_set = TableSet::new();
        let mut condition = gen_parse_result!(ConditionExpr::parse, "1 < 0 or 1 = 2");
        assert_ok!(check_condition(&mut condition, &table_set, &None));

        let mut condition = gen_parse_result!(ConditionExpr::parse, "1 = null");
        assert_err!(check_condition(&mut condition, &table_set, &None), CompileErrorType::SemInvalidValueType);

        let mut condition = gen_parse_result!(ConditionExpr::parse, "1 < \"i am string\"");
        assert_err!(check_condition(&mut condition, &table_set, &None), CompileErrorType::SemInvalidValueType);
        
        let mut condition = gen_parse_result!(ConditionExpr::parse, "\"aaa\" = \"bbb\"");
        assert_ok!(check_condition(&mut condition, &table_set, &None));

        add_table(&mut table_set);
        let mut condition = gen_parse_result!(ConditionExpr::parse, "author_id is not null");
        assert_ok!(check_condition(&mut condition, &table_set, &None));

        let mut condition = gen_parse_result!(ConditionExpr::parse, "author_id is 1");
        assert_err!(check_condition(&mut condition, &table_set, &None), CompileErrorType::SemInvalidValueType);

        let mut condition = gen_parse_result!(ConditionExpr::parse, "2 is null");
        assert_err!(check_condition(&mut condition, &table_set, &None), CompileErrorType::SemInvalidValueType);
    }
    {// attirbute check
        let mut table_set = TableSet::new();
        let mut condition = gen_parse_result!(ConditionExpr::parse, "a is null");
        assert_err!(check_condition(&mut condition, &table_set, &None), CompileErrorType::SemInvalidAttribute);

        add_table(&mut table_set);
        let mut condition = gen_parse_result!(ConditionExpr::parse, "author_id is null");
        assert_ok!(check_condition(&mut condition, &table_set, &None));

        let mut condition = gen_parse_result!(ConditionExpr::parse, "book.author_id is null");
        assert_ok!(check_condition(&mut condition, &table_set, &None));

        let mut condition = gen_parse_result!(ConditionExpr::parse, "book.id > 1");
        assert_ok!(check_condition(&mut condition, &table_set,
            &Some((Some("book".to_string()), "id".to_string()))));
        assert_err!(check_condition(&mut condition, &table_set,
            &Some((Some("book".to_string()), "author_id".to_string()))),
            CompileErrorType::SemShouldUseGroupByAttribute);

        let mut condition = gen_parse_result!(ConditionExpr::parse, "sum(book.id) > 1");
        assert_ok!(check_condition(&mut condition, &table_set,
            &Some((Some("book".to_string()), "id".to_string()))));

        let mut condition = gen_parse_result!(ConditionExpr::parse, "invalid_func(book.id) > 1");
        assert_err!(check_condition(&mut condition, &table_set,
            &Some((Some("book".to_string()), "id".to_string()))),
            CompileErrorType::SemInvalidAggreFuncName);

        let mut condition = gen_parse_result!(ConditionExpr::parse, "sum(book.id) > 1");
        assert_err!(check_condition(&mut condition, &table_set, &None),
            CompileErrorType::SemInvalidAggregateFunctionUse);

        let mut condition = gen_parse_result!(ConditionExpr::parse, "book.name > 0");
        assert_err!(check_condition(&mut condition, &table_set, &None), CompileErrorType::SemInvalidValueType);

        let mut condition = gen_parse_result!(ConditionExpr::parse, "author.name is null");
        assert_err!(check_condition(&mut condition, &table_set, &None),
            CompileErrorType::SemAttributeNotNullable);

        let mut condition = gen_parse_result!(ConditionExpr::parse, "id is null");
        assert_err!(check_condition(&mut condition, &table_set, &None),
            CompileErrorType::SemInvalidAttribute);
    }
}

#[test]
fn test_check_insert() {
    let mut table_set = TableSet::new();
    add_table(&mut table_set);
    let mut insert = gen_parse_result!(InsertStatement::parse, "insert book values(1, 2, \"book name\")");
    assert_ok!(check_insert(&mut insert, &table_set));

    let mut insert = gen_parse_result!(InsertStatement::parse, "insert book values(1, 2, \"book name\", 3)");
    assert_err!(check_insert(&mut insert, &table_set), CompileErrorType::SemInvalidInsertValuesNum);

    let mut insert = gen_parse_result!(InsertStatement::parse, "insert book values(1, 2.0, \"book name\")");
    assert_err!(check_insert(&mut insert, &table_set), CompileErrorType::SemInvalidInsertValueType);

    let mut insert = gen_parse_result!(InsertStatement::parse, "insert author values(1, \"doyoubi\")");
    assert_ok!(check_insert(&mut insert, &table_set));
    let mut insert = gen_parse_result!(InsertStatement::parse,
        "insert author values(1, \"it is difficult to come up with a long name\")");
    assert_err!(check_insert(&mut insert, &table_set), CompileErrorType::SemInvalidInsertCharLen);

    let mut insert = gen_parse_result!(InsertStatement::parse, "insert author values(1, null)");
    assert_err!(check_insert(&mut insert, &table_set), CompileErrorType::SemAttributeNotNullable);

    let mut insert = gen_parse_result!(InsertStatement::parse, "insert book values(1, null, \"book name\")");
    assert_ok!(check_insert(&mut insert, &table_set));
}

#[test]
fn test_check_update() {
    let mut table_set = TableSet::new();
    add_table(&mut table_set);

    let mut update = gen_parse_result!(UpdateStatement::parse,
        "update book set author_id = 2, name = \"doyoubi\" where book.id = 1");
    assert_ok!(check_update(&mut update, &table_set));

    let mut update = gen_parse_result!(UpdateStatement::parse, "update book set id = 1");
    assert_err!(check_update(&mut update, &table_set), CompileErrorType::SemChangePrimaryAttr);

    let mut update = gen_parse_result!(UpdateStatement::parse, "update book set invalid_attr = 1");
    assert_err!(check_update(&mut update, &table_set), CompileErrorType::SemInvalidAttribute);

    let mut update = gen_parse_result!(UpdateStatement::parse, "update book set author_id = 2.2");
    assert_err!(check_update(&mut update, &table_set), CompileErrorType::SemInvalidInsertValueType);

    let mut update = gen_parse_result!(UpdateStatement::parse, "update author set name = null");
    assert_err!(check_update(&mut update, &table_set), CompileErrorType::SemAttributeNotNullable);
}

#[test]
fn test_check_select() {
    let mut table_set = TableSet::new();
    add_table(&mut table_set);

    let mut select = gen_parse_result!(SelectStatement::parse,
        "select book.id, book.name from book where book.id > 1");
    assert_ok!(check_select(&mut select, &table_set));

    let mut select = gen_parse_result!(SelectStatement::parse,
        "select author_id, min(book.author_id) from book where book.id > 1 \
        group by author_id having min(author_id) > 2 and author_id > 3");
    assert_ok!(check_select(&mut select, &table_set));

    let mut select = gen_parse_result!(SelectStatement::parse,
        "select book.author_id, min(book.author_id) from book where book.id > 1 \
        group by author_id having min(book.id) > 2 and book.id > 3");
    assert_err!(check_select(&mut select, &table_set), CompileErrorType::SemShouldUseGroupByAttribute);

    let mut select = gen_parse_result!(SelectStatement::parse,
        "select book.id, min(book.id) from book where book.id > 1 \
        group by author_id having min(author_id) > 2 and author_id > 3");
    assert_err!(check_select(&mut select, &table_set), CompileErrorType::SemShouldUseGroupByAttribute);

    let mut select = gen_parse_result!(SelectStatement::parse, "select * from book where num = 1");
    assert_err!(check_select(&mut select, &table_set), CompileErrorType::SemInvalidAttribute);

    let mut select = gen_parse_result!(SelectStatement::parse, "select * from book where num = 1");
    assert_err!(check_select(&mut select, &table_set), CompileErrorType::SemInvalidAttribute);

    let mut select = gen_parse_result!(SelectStatement::parse, "select * from book group by book.name");
    assert_err!(check_select(&mut select, &table_set), CompileErrorType::SemSelectAllWithGroupBy);

    let mut select = gen_parse_result!(SelectStatement::parse, "select num from book");
    assert_err!(check_select(&mut select, &table_set), CompileErrorType::SemInvalidAttribute);

    let mut select = gen_parse_result!(SelectStatement::parse, "select book.name from book order by book.id");
    assert_ok!(check_select(&mut select, &table_set));

    let mut select = gen_parse_result!(SelectStatement::parse, "select book.name from book order by num");
    assert_err!(check_select(&mut select, &table_set), CompileErrorType::SemInvalidAttribute);

    let mut select = gen_parse_result!(SelectStatement::parse,
        "select book.name from book group by book.name order by book.id");
    assert_err!(check_select(&mut select, &table_set), CompileErrorType::SemShouldUseGroupByAttribute);

    let mut select = gen_parse_result!(SelectStatement::parse,
        "select book.id from book group by book.name order by book.name");
    assert_err!(check_select(&mut select, &table_set), CompileErrorType::SemShouldUseGroupByAttribute);
}

#[test]
fn test_select_table_not_exist() {
    let table_set = TableSet::new();
    let mut select = gen_parse_result!(SelectStatement::parse,
        "select msg.id from msg");
    assert_err!(check_select(&mut select, &table_set), CompileErrorType::SemTableNotExist);
}

#[test]
fn test_complete_table_in_attribute() {
    let mut table_set = TableSet::new();
    add_table(&mut table_set);

    let mut select = gen_parse_result!(SelectStatement::parse,
        "select author_id, min(author_id) from book where book.id > 1 \
        group by author_id having min(author_id) > 2 and author_id > 3");
    assert_ok!(check_select(&mut select, &table_set));
    assert_eq!(format!("{}", select), "select (book.author_id), min(book.author_id) from book \
        where ((book.id) > Integer(1)) group by (book.author_id) \
        having ((min(book.author_id) > Integer(2)) and ((book.author_id) > Integer(3)))");
}
