#ifndef BLT_LEXER_H
#define BLT_LEXER_H

#include <vector>
#include <string>
#include <memory>

#include "compile_error.h"

namespace blt
{

enum class TokenType
{
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
    UnKnown,
};


struct Token
{
    typedef std::shared_ptr<Token> Ptr;
    typedef std::vector<Ptr> List;
    typedef List::iterator Iter;

    size_t column;
    std::string value;
    TokenType type;

    Token(size_t _column, std::string _value, TokenType _type)
        : column(_column), value(_value), type(_type)
    {}
};


struct CompileError
{
    typedef std::vector<CompileError> List;

    CompileErrorType errorType;
    Token::Ptr token;
    std::string errorMsg;
};


struct TokenLine
{
    typedef std::shared_ptr<TokenLine> Ptr;

    Token::List tokens;
    CompileError::List errors;

    static Ptr parse(const std::string & codeString);
    bool checkUnEscapeString(const std::string & s, Token::Ptr & token);
};

}

#endif
