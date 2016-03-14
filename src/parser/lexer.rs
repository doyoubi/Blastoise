extern crate std;
use std::rc::Rc;
use std::option::Option::{Some, None};
use std::result::Result;
use std::result::Result::{Ok, Err};
use std::slice::Iter;
use ::parser::compile_error::{CompileError, CompileErrorType, ErrorList, ErrorRef};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TokenType {
    IntegerLiteral,
    FloatLiteral,
    StringLiteral,
    Identifier, // for table, attribute, alias

    Select,
    From,
    Where,
    Order,
    By,
    Group,
    Having,

    Insert,
    Values,
    Update,
    Set,
    Delete,

    Create,
    Table,
    Drop,

    Int,
    Float,
    Char,
    Primary,

    Null,         // null
    OpenBracket,  // (
    CloseBracket, // )
    Comma,        // ,
    Add,          // +
    Sub,          // -
    Star,         // *, for wildcard and multiplication
    Div,          // /
    Mod,          // %
    LT,           // <
    GT,           // >
    LE,           // <=
    GE,           // >=
    EQ,           // =
    NE,           // !=
    GetMember,    // .
    And,          // and
    Or,           // or
    Not,          // not
    Is,           // is
    IsNot,        // is not
    UnKnown,
}

#[derive(Debug)]
pub struct Token
{
    pub column : i32,
    pub value : String,
    pub token_type : TokenType,
}

pub type TokenRef = Rc<Token>;
pub type TokenList = std::vec::Vec<TokenRef>;
pub type TokenIter<'a> = Iter<'a, TokenRef>;

#[derive(Clone)]
pub struct TokenLine
{
    pub tokens : TokenList,
    pub errors : ErrorList,
}

#[derive(Copy, Clone)]
enum State
{
    Begin,
    InInteger,
    InFloat,
    InString,
    InStringEscaping,
    InIdentifier,
}

impl TokenLine {
    pub fn parse(code_string : &str) -> TokenLine {
        let mut line = TokenLine{ tokens : TokenList::new(), errors : ErrorList::new() };
        if let Err(err) = check_ascii(code_string) {
            line.errors.push(err);
            return line; 
        }

        let mut state = State::Begin;
        let tail = code_string.len();
        let head_unused_tag = -1;
        let mut head = head_unused_tag;

        let add_token = |value : String,
                         token_type : TokenType,
                         head : i32,
                         i : i32,
                         line : &mut TokenLine| {
            let token_head = if head == head_unused_tag { i } else { head };
            let mut token = Token{
                column : token_head + 1,
                value : value.clone(),
                token_type : token_type
            };
            if let TokenType::Identifier = token_type {
                let t = str_to_token_type(&*value.to_lowercase());
                if let Some(keyword_type) = t {
                    token.token_type = keyword_type;
                };
            } else if let TokenType::StringLiteral = token_type {
                let unescaped_str = get_unescaped_string(&value);
                match unescaped_str {
                    Some(unescaped_str) => token.value = unescaped_str,
                    // treat it as an valid string, but without escape, raise an error and go on
                    None => {
                        let token_rc = Rc::new(token);
                        line.errors.push(Rc::new(CompileError{
                            error_type : CompileErrorType::LexerInvalidEscapeChar,
                            token : token_rc.clone(),
                            error_msg : "invalid escape char found in string literal".to_string()
                        }));
                        line.tokens.push(token_rc);
                        return;
                    }
                };
            }
            if let TokenType::Not = token.token_type {
                if let Some(TokenType::Is) = line.tokens.last().map(|token| token.token_type) {
                    token.token_type = TokenType::IsNot;
                    token.value = "is not".to_string();
                    token.column = line.tokens.last().unwrap().column;
                    line.tokens.pop();
                }
            }
            line.tokens.push(Rc::new(token));
        };

        let add_error = |error_type : CompileErrorType,
                         value : String,
                         error_msg : String,
                         head : i32,
                         i : i32,
                         line : &mut TokenLine| {
            let token_head = if head == head_unused_tag { i } else { head };
            let token = Token{
                column : token_head + 1,
                value : value,
                token_type : TokenType::UnKnown
            };
            let error = CompileError{
                error_type : error_type,
                token : Rc::new(token),
                error_msg : error_msg
            };
            line.errors.push(Rc::new(error));
        };

        let mut it = code_string.chars().enumerate().peekable();
        loop {
            let tmp = it.clone();
            let (i, c) = match it.next() {
                Some((i, c)) => (i as i32, c),
                None => (tail as i32, '\0'),
            };
            let next_c = match it.peek() {
                Some(&(_, next_c)) => next_c,
                None => '\0',
            };
            match state {
                State::Begin => {
                    if is_ignore_char(c) {
                        ()
                    } else if let Some(token_type) = convert_two_char_token(c, next_c) {
                        add_token([c, next_c].iter().cloned().collect(), token_type,
                            head, i, &mut line);
                        it.next();
                    } else if let Some(token_type) = convert_single_char_token(c) {
                        add_token(c.to_string(), token_type, head, i, &mut line);
                    } else if c == '\"' {
                         state = State::InString;
                         head = i;
                    } else if let '0' ... '9' = c {
                        state = State::InInteger;
                        head = i;
                    } else if is_identifier_first_char(c) {
                        state = State::InIdentifier;
                        head = i;
                    } else {
                        add_error(CompileErrorType::LexerUnexpectedChar, c.to_string(),
                            format!("illegal char found: '{}'", c), head, i, &mut line);
                    }
                }
                State::InIdentifier => {
                    if is_identifier_char(c) {
                        () // go on
                    } else {
                        add_token((&code_string[head as usize .. i as usize]).to_string(),
                            TokenType::Identifier,
                            head, i, &mut line);
                        head = head_unused_tag;
                        state = State::Begin;
                        it = tmp;  // let next loop handle separator
                    }
                }
                State::InString => {
                    match c{
                        '\n' | '\0' => {
                            add_error(CompileErrorType::LexerInCompleteString,
                                (&code_string[head as usize .. i as usize]).to_string(),
                                "incomplete string, string must be closed with '\"'".to_string(),
                                head, i, &mut line);
                            head = head_unused_tag;
                            state = State::Begin;
                            it = tmp;
                        }
                        '\\' => state = State::InStringEscaping,
                        '\"' => {
                            add_token((&code_string[(head+1) as usize .. i as usize]).to_string(),
                                TokenType::StringLiteral,
                                head, i, &mut line);
                            head = head_unused_tag;
                            state = State::Begin;
                        }
                        _ => (),  // go on
                    }
                }
                State::InStringEscaping => {
                    match c {
                        '\n' | '\0' => {
                            add_error(CompileErrorType::LexerInCompleteString,
                                (&code_string[head as usize .. i as usize]).to_string(),
                                "incomplete string, string must be closed with '\"'".to_string(),
                                head, i, &mut line);
                            head = head_unused_tag;
                            state = State::Begin;
                            it = tmp;
                        }
                        _ => {
                            // not handle escape char here, just let it there
                            state = State::InString;
                        }
                    }
                }
                State::InInteger => {
                    match c {
                        '0' ... '9' => (),  // go on
                        '.' => {
                            if let '0' ... '9' = next_c {
                                state = State::InFloat;
                            } else {
                                // ignore this '.' and treat this token as Float, but raise error
                                add_token((&code_string[head as usize .. i as usize]).to_string(),
                                    TokenType::FloatLiteral, head, i, &mut line);
                                add_error(CompileErrorType::LexerInvalidFloat,
                                    (&code_string[head as usize .. (i+1) as usize]).to_string(),
                                    "'.' should be followed by digit".to_string(),
                                    head, i, &mut line);
                                state = State::Begin;
                                head = head_unused_tag;
                            };
                        }
                        _ => {
                            add_token((&code_string[head as usize .. i as usize]).to_string(),
                                TokenType::IntegerLiteral, head, i, &mut line);
                            state = State::Begin;
                            head = head_unused_tag;
                            it = tmp;
                        }
                    }
                }
                State::InFloat => {
                    match c {
                        '0' ... '9' => (),  // go on
                        _ => {
                            add_token((&code_string[head as usize .. i as usize]).to_string(),
                                    TokenType::FloatLiteral, head, i, &mut line);
                            state = State::Begin;
                            head = head_unused_tag;
                            it = tmp;
                        }
                    }
                }
            }  // end of match
            if c == '\0' {
                break;
            }
        } // end of while

        line
    }
}

fn check_ascii(input : &str) -> Result<(), ErrorRef> {
    for c in input.chars() {
        let n = c as i32;
        if 0 <= n && n < 128 {continue;}
        return Err(Rc::new(CompileError{
            error_type : CompileErrorType::LexerInvalidAscii,
            error_msg : format!("invalid ascii char: {}", c),
            token : Rc::new(Token{
                column : 0,
                value : "".to_string(),
                token_type : TokenType::UnKnown
            }),  // dummy token
        }));
    }
    Ok(())
}

fn get_unescaped_string(s : &str) -> Option<String> {
    let mut unescaped_str = String::new();
    let mut escaping = false;
    for (i, c) in s.chars().enumerate() {
        if escaping {
            let unescaped_char = match c {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\\' => '\\',
                '\'' => '\'',
                '"' => '\"',
                '0' => '\0',
                _ => return None,
            };
            unescaped_str.push(unescaped_char);
            escaping = false;
        }
        else {
            if c == '\\' {
                if i + 1 == s.len()
                { return None; }
                escaping = true;
            }
            else
            { unescaped_str.push(c); }
        };
    };
    Some(unescaped_str)
}

fn str_to_token_type(s : &str) -> Option<TokenType> {
    match s {
        "select" => Some(TokenType::Select),
        "from"   => Some(TokenType::From),
        "where"  => Some(TokenType::Where),
        "order"  => Some(TokenType::Order),
        "by"     => Some(TokenType::By),
        "group"  => Some(TokenType::Group),
        "having" => Some(TokenType::Having),
        "insert" => Some(TokenType::Insert),
        "values" => Some(TokenType::Values),
        "update" => Some(TokenType::Update),
        "set"    => Some(TokenType::Set),
        "delete" => Some(TokenType::Delete),
        "create" => Some(TokenType::Create),
        "table"  => Some(TokenType::Table),
        "drop"   => Some(TokenType::Drop),
        "null"   => Some(TokenType::Null),
        "and"    => Some(TokenType::And),
        "or"     => Some(TokenType::Or),
        "not"    => Some(TokenType::Not),
        "is"     => Some(TokenType::Is),
        "int"    => Some(TokenType::Int),
        "float"  => Some(TokenType::Float),
        "char"   => Some(TokenType::Char),
        "primary"=> Some(TokenType::Primary),
        _ => None,
    }
}

fn is_ignore_char(c : char) -> bool {
    match c {
        '\n' | '\0' | '\t' | '\r' | ' ' => true,
        _ => false,
    }
}

fn convert_two_char_token(c : char, next_c : char) -> Option<TokenType> {
    match (c, next_c) {
        ('!', '=') => Some(TokenType::NE),
        ('<', '=') => Some(TokenType::LE),
        ('>', '=') => Some(TokenType::GE),
        _ => None,
    }
}

fn convert_single_char_token(c : char) -> Option<TokenType> {
    match c {
        '(' => Some(TokenType::OpenBracket),
        ')' => Some(TokenType::CloseBracket),
        ',' => Some(TokenType::Comma),
        '+' => Some(TokenType::Add),
        '-' => Some(TokenType::Sub),
        '*' => Some(TokenType::Star),
        '/' => Some(TokenType::Div),
        '%' => Some(TokenType::Mod),
        '<' => Some(TokenType::LT),
        '>' => Some(TokenType::GT),
        '=' => Some(TokenType::EQ),
        '.' => Some(TokenType::GetMember),
        _ => None,
    }
}

fn is_identifier_first_char(c : char) -> bool {
    match c {
        'a' ... 'z' | 'A' ... 'Z' | '_' => true,
        _ => false,
    }
}

fn is_identifier_char(c : char) -> bool {
    match c {
        'a' ... 'z' | 'A' ... 'Z' | '_' | '0' ... '9' => true,
        _ => false,
    }
}
