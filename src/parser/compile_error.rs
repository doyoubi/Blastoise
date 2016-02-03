use std::rc::Rc;
use std::vec::Vec;


#[allow(dead_code)]  // lint bug
#[derive(Eq, PartialEq, Debug)]
pub enum CompileErrorType {
    LexerInvalidEscapeChar,
    LexerUnexpectedChar,
    LexerInCompleteString,
    LexerInvalidFloat,

    ParserNoMoreToken,
    ParserUnExpectedTokenType,
    ParserTableAttrNotExist,
    ParserNoTable,
    ParserLackOfSpecifyingTable,
}

pub struct CompileError {
    pub error_type : CompileErrorType,
    pub token : Rc<::parser::lexer::Token>,
    pub error_msg : String,
}

pub type ErrorList = Vec<Rc<CompileError>>;
