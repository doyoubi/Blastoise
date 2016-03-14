use std::rc::Rc;
use std::iter::Iterator;
use std::vec::IntoIter;
use ::parser::lexer::{Token, TokenLine, TokenType};
use ::parser::compile_error::CompileErrorType;

#[allow(dead_code)]
struct TokenTestHelper
{
    token_line : TokenLine,
    iter : IntoIter<Rc<Token>>,
}

impl TokenTestHelper {
    #[allow(dead_code)]
    fn new(code_string : &str) -> TokenTestHelper {
        let line = TokenLine::parse(code_string);
        TokenTestHelper{
            token_line : line.clone(),
            iter : line.tokens.into_iter(),
        }
    }

    #[allow(dead_code)]
    fn next(&mut self) -> Option<Rc<Token>> {
        self.iter.next()
    }

    #[allow(dead_code)]
    fn get_errors(&self) -> &::parser::compile_error::ErrorList {
        &self.token_line.errors
    }
}

macro_rules! assert_token_eq {
    ($helper:expr, $column:expr, $value:expr, $token_type:expr) => ({
            let t = $helper.next();
            assert!(t.is_some());
            if let Some(token) = t {
                assert_eq!(token.column, $column);
                assert_eq!(token.value, $value);
                assert_eq!(token.token_type, $token_type);
            };
        });
}

macro_rules! assert_error_eq {
    ($helper:expr, $index:expr, $error_type:expr) => ({
        let err = &$helper.token_line.errors[$index];
        assert_eq!(err.error_type, $error_type);
    })
}

macro_rules! assert_token_len {
    ($helper:expr, $len:expr) => (assert_eq!($helper.token_line.tokens.len(), $len);)
}

macro_rules! assert_error_len {
    ($helper:expr, $len:expr) => (assert_eq!($helper.token_line.errors.len(), $len);)
}

#[test]
fn test_empty_string() {
    let h = TokenTestHelper::new("");
    assert_token_len!(h, 0);
    assert_error_len!(h, 0);
}

#[test]
fn test_integer_token() {
    let mut h = TokenTestHelper::new("1 233 6666");
    assert_token_len!(h, 3);
    assert_error_len!(h, 0);
    assert_token_eq!(h, 1, "1", TokenType::IntegerLiteral);
    assert_token_eq!(h, 3, "233", TokenType::IntegerLiteral);
    assert_token_eq!(h, 7, "6666", TokenType::IntegerLiteral);
}

#[test]
fn test_float_token() {
    let mut h = TokenTestHelper::new("1.0 2.333 12.");
    assert_token_len!(h, 3);
    assert_error_len!(h, 1);
    assert_token_eq!(h, 1, "1.0", TokenType::FloatLiteral);
    assert_token_eq!(h, 5, "2.333", TokenType::FloatLiteral);
    assert_token_eq!(h, 11, "12", TokenType::FloatLiteral);
    assert_error_eq!(h, 0, CompileErrorType::LexerInvalidFloat);
}

#[test]
fn test_invalid_char() {
    let mut h = TokenTestHelper::new("1$2##3");
    assert_token_len!(h, 3);
    assert_error_len!(h, 3);
    assert_token_eq!(h, 1, "1", TokenType::IntegerLiteral);
    assert_token_eq!(h, 3, "2", TokenType::IntegerLiteral);
    assert_token_eq!(h, 6, "3", TokenType::IntegerLiteral);
    assert_error_eq!(h, 0, CompileErrorType::LexerUnexpectedChar);
    assert_error_eq!(h, 1, CompileErrorType::LexerUnexpectedChar);
    assert_error_eq!(h, 2, CompileErrorType::LexerUnexpectedChar);
}

#[test]
fn test_string_token() {
    let mut h = TokenTestHelper::new(
        "\"a\" \"str1\"\"str2\"\
         \"\\r\\t\\\\ \\' \\\" \"\
         \"unfinished escape \\j end\"\
         \"incomplete string"
        );
    assert_token_len!(h, 5);
    assert_error_len!(h, 2);
    assert_token_eq!(h, 1, "a", TokenType::StringLiteral);
    assert_token_eq!(h, 5, "str1", TokenType::StringLiteral);
    assert_token_eq!(h, 11, "str2", TokenType::StringLiteral);
    assert_token_eq!(h, 17, "\r\t\\ ' \" ", TokenType::StringLiteral);
    assert_token_eq!(h, 32, "unfinished escape \\j end", TokenType::StringLiteral);
    assert_error_eq!(h, 0, CompileErrorType::LexerInvalidEscapeChar);
    assert_error_eq!(h, 1, CompileErrorType::LexerInCompleteString);
}

#[test]
fn test_identifier_token() {
    let mut h = TokenTestHelper::new("ident ident2 _233");
    assert_token_len!(h, 3);
    assert_error_len!(h, 0);
    assert_token_eq!(h, 1, "ident", TokenType::Identifier);
    assert_token_eq!(h, 7, "ident2", TokenType::Identifier);
    assert_token_eq!(h, 14, "_233", TokenType::Identifier);
}

#[test]
fn test_keyword_token() {
    let mut h = TokenTestHelper::new(
        "select fROM Where order by group having \
         insert values update set delete \
         create table drop null and or not is \
         int float char"
        );
    assert_token_len!(h, 23);
    assert_error_len!(h, 0);
    assert_token_eq!(h, 1, "select", TokenType::Select);
    assert_token_eq!(h, 8, "fROM", TokenType::From);
    assert_token_eq!(h, 13, "Where", TokenType::Where);
    assert_token_eq!(h, 19, "order", TokenType::Order);
    assert_token_eq!(h, 25, "by", TokenType::By);
    assert_token_eq!(h, 28, "group", TokenType::Group);
    assert_token_eq!(h, 34, "having", TokenType::Having);
    assert_token_eq!(h, 40 + 1, "insert", TokenType::Insert);
    assert_token_eq!(h, 40 + 8, "values", TokenType::Values);
    assert_token_eq!(h, 40 + 15, "update", TokenType::Update);
    assert_token_eq!(h, 40 + 22, "set", TokenType::Set);
    assert_token_eq!(h, 40 + 26, "delete", TokenType::Delete);
    assert_token_eq!(h, 40 + 32 + 1, "create", TokenType::Create);
    assert_token_eq!(h, 40 + 32 + 8, "table", TokenType::Table);
    assert_token_eq!(h, 40 + 32 + 14, "drop", TokenType::Drop);
    assert_token_eq!(h, 40 + 32 + 19, "null", TokenType::Null);
    assert_token_eq!(h, 40 + 32 + 24, "and", TokenType::And);
    assert_token_eq!(h, 40 + 32 + 28, "or", TokenType::Or);
    assert_token_eq!(h, 40 + 32 + 31, "not", TokenType::Not);
    assert_token_eq!(h, 40 + 32 + 35, "is", TokenType::Is);
    assert_token_eq!(h, 40 + 32 + 37 + 1, "int", TokenType::Int);
    assert_token_eq!(h, 40 + 32 + 37 + 5, "float", TokenType::Float);
    assert_token_eq!(h, 40 + 32 + 37 + 11, "char", TokenType::Char);
}

#[test]
fn test_operator_token() {
    let mut h = TokenTestHelper::new("(),+-*/%<><=>==!=.");
    assert_token_len!(h, 15);
    assert_error_len!(h, 0);
    assert_token_eq!(h, 1, "(", TokenType::OpenBracket);
    assert_token_eq!(h, 2, ")", TokenType::CloseBracket);
    assert_token_eq!(h, 3, ",", TokenType::Comma);
    assert_token_eq!(h, 4, "+", TokenType::Add);
    assert_token_eq!(h, 5, "-", TokenType::Sub);
    assert_token_eq!(h, 6, "*", TokenType::Star);
    assert_token_eq!(h, 7, "/", TokenType::Div);
    assert_token_eq!(h, 8, "%", TokenType::Mod);
    assert_token_eq!(h, 9, "<", TokenType::LT);
    assert_token_eq!(h, 10, ">", TokenType::GT);
    assert_token_eq!(h, 11, "<=", TokenType::LE);
    assert_token_eq!(h, 13, ">=", TokenType::GE);
    assert_token_eq!(h, 15, "=", TokenType::EQ);
    assert_token_eq!(h, 16, "!=", TokenType::NE);
    assert_token_eq!(h, 18, ".", TokenType::GetMember);
}

#[test]
fn test_is_not() {
    let mut h = TokenTestHelper::new("is not");
    assert_token_len!(h, 1);
    assert_error_len!(h, 0);
    assert_token_eq!(h, 1, "is not", TokenType::IsNot);
}

#[test]
fn test_ascii() {
    let h = TokenTestHelper::new("select 光星 from 深大");
    assert_error_len!(h, 1);
    assert_error_eq!(h, 0, CompileErrorType::LexerInvalidAscii);
}
