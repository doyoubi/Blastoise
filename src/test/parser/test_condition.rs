use std::option::Option::None;
use std::result::Result::{Ok, Err};
use ::parser::condition::ArithExpr;
use ::parser::common::ValueType;
use ::parser::compile_error::CompileErrorType;


macro_rules! test_literal {
    ($input_str:expr, $value:expr, $token_type:pat) => ({
        let tokens = gen_token!($input_str);
        assert_eq!(tokens.len(), 1);
        let mut it = tokens.iter();
        let arith_exp = ArithExpr::parse_arith_operant(&mut it);
        assert_pattern!(arith_exp, Ok(_));
        let arith_exp = arith_exp.unwrap();
        let (value, value_type) = extract!(
            arith_exp, ArithExpr::ValueExpr{ value, value_type }, (value, value_type));
        assert_eq!(value, $value);
        assert_pattern!(value_type, $token_type);
        assert_pattern!(it.next(), None);
    });
}

#[test]
fn test_parse_arith_operant() {
    test_literal!("233", "233", ValueType::Integer);
    test_literal!("233.666", "233.666", ValueType::Float);
    test_literal!("\"identifier\"", "identifier", ValueType::String);
    test_literal!("null", "null", ValueType::Null);
    {
        let tokens = gen_token!("+");
        assert_eq!(tokens.len(), 1);
        let mut it = tokens.iter();
        let invalid_exp = ArithExpr::parse_arith_operant(&mut it);
        assert_pattern!(invalid_exp, Err(_));
        let errs = invalid_exp.unwrap_err();
        assert_eq!(errs.len(), 1);
        let ref err = errs[0];
        assert_pattern!(err.error_type, CompileErrorType::ParserUnExpectedTokenType);
    }
}
