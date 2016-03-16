use ::parser::unimpl::check_cond;
use ::parser::condition::{ConditionExpr, ArithExpr, CmpOperantExpr, CmpOp};


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
