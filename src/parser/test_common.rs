use std::vec::Vec;
use super::lexer::{Token, TokenList, TokenType, TokenIter};
// use ::store::table::{TableRef, AttrRef};
use super::common::{
    check_single_token_type,
};


#[test]
fn test_check_single_token_type() {
    {
        // test empty token line
        let line = TokenList::new();
        let it = line.iter();
        let errs = check_single_token_type(TokenType::Null);
        assert!(errs.is_some());
        let errs = errs.unwrap();
        assert_eq!(errs.len(), 1);
    }
}