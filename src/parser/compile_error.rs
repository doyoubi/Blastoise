use std::rc::Rc;

#[derive(Eq, PartialEq, Debug)]
pub enum CompileErrorType {
    LexerInvalidEscapeChar,
    LexerUnexpectedChar,
    LexerInCompleteString,
    LexerInvalidFloat,
}

pub struct CompileError {
    pub error_type : CompileErrorType,
    pub token : Rc<::parser::lexer::Token>,
    pub error_msg : String,
}
