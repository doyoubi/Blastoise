use ::parser::condition::{ConditionExpr, ArithExpr, CmpOperantExpr, CmpOp};
use ::parser::select::SelectStatement;
use ::parser::unimpl::{
    check_cond,
    check_select,
};


#[test]
fn test_check_cond() {
    let condition = gen_parse_result!(ConditionExpr::parse, "1 < 0 or 1 = 2");
    assert_pattern!(check_cond(&condition), Ok(..));

    let condition = gen_parse_result!(ConditionExpr::parse, "a is null");
    assert_pattern!(check_cond(&condition), Err(..));

    let condition = gen_parse_result!(ConditionExpr::parse, "1 < 0 or not a is null");
    assert_pattern!(check_cond(&condition), Err(..));

    let condition = gen_parse_result!(ConditionExpr::parse, "a is null or 1 < 0");
    assert_pattern!(check_cond(&condition), Err(..));

    let condition = gen_parse_result!(ConditionExpr::parse, "1 is null or 1 < 0");
    assert_pattern!(check_cond(&condition), Err(..));
}

#[test]
fn test_check_select() {
    let select = gen_parse_result!(SelectStatement::parse,
        "select * from msg where a > 1");
    assert_pattern!(check_select(&select), Ok(..));

    let select = gen_parse_result!(SelectStatement::parse,
        "select sum(a) from msg");
    assert_pattern!(check_select(&select), Err(..));

    let select = gen_parse_result!(SelectStatement::parse,
        "select sum(a) from msg");
    assert_pattern!(check_select(&select), Err(..));

    let select = gen_parse_result!(SelectStatement::parse,
        "select a from msg group by a");
    assert_pattern!(check_select(&select), Err(..));

    let select = gen_parse_result!(SelectStatement::parse,
        "select a from msg order by a");
    assert_pattern!(check_select(&select), Err(..));

    let select = gen_parse_result!(SelectStatement::parse,
        "select a from (select b from msg)");
    assert_pattern!(check_select(&select), Err(..));

    let select = gen_parse_result!(SelectStatement::parse,
        "select * from msg where a is null");
    assert_pattern!(check_select(&select), Err(..));

    let select = gen_parse_result!(SelectStatement::parse,
        "select * from msg, book");
    assert_pattern!(check_select(&select), Err(..));
}
