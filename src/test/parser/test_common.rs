use std::option::Option::{Some, None};
use std::result::Result::Ok;
use ::parser::common::{
    check_parse_to_end,
    align_iter,
    Statement,
};


#[test]
fn test_check_parse_to_end() {
    {
        let tokens = gen_token!("");
        assert_eq!(tokens.len(), 0);
        let it = tokens.iter();
        let res = check_parse_to_end(&it);
        assert_pattern!(res, None);
    }
    {
        let tokens = gen_token!("un_parsed_token");
        assert_eq!(tokens.len(), 1);
        let it = tokens.iter();
        let res = check_parse_to_end(&it);
        assert_pattern!(res, Some(_));
    }
}

#[test]
fn test_align_iter() {
    let tokens = gen_token!("1 2 3");
    assert_eq!(tokens.len(), 3);
    let mut i = tokens.iter();
    let mut j = tokens.iter();
    j.next();
    assert_eq!(i.len(), 3);
    assert_eq!(j.len(), 2);
    align_iter(&mut i, &mut j);
    assert_eq!(i.len(), j.len())
}

fn gen_stmt(s : &str) -> Statement {
    let tokens = gen_token!(s);
    extract!(Statement::parse(&mut tokens.iter()), Ok(stmt), stmt)
}

#[test]
fn test_statement() {
    {
        let stmt = gen_stmt("select * from book");
        let stmt = extract!(stmt, Statement::Select(stmt), stmt);
        assert_eq!(format!("{}", stmt), "select * from book");
    }
    {
        let stmt = gen_stmt("update book set price = 100");
        let stmt = extract!(stmt, Statement::Update(stmt), stmt);
        assert_eq!(format!("{}", stmt), "update book set (price = Integer(100))");
    }
    {
        let stmt = gen_stmt("insert book values(100)");
        let stmt = extract!(stmt, Statement::Insert(stmt), stmt);
        assert_eq!(format!("{}", stmt), "insert book values(Integer(100))");
    }
    {
        let stmt = gen_stmt("delete from book");
        let stmt = extract!(stmt, Statement::Delete(stmt), stmt);
        assert_eq!(format!("{}", stmt), "delete from book");
    }
    {
        let stmt = gen_stmt("create table book(name char(100))");
        let stmt = extract!(stmt, Statement::Create(stmt), stmt);
        assert_eq!(format!("{}", stmt), "create table book ((name Char(100) null))");
    }
    {
        let stmt = gen_stmt("drop table book");
        let stmt = extract!(stmt, Statement::Drop(stmt), stmt);
        assert_eq!(format!("{}", stmt), "drop table book");
    }
}
