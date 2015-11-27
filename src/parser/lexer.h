#ifndef BLT_LEXER_H
#define BLT_LEXER_H

namespace blt
{

enum class Token
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

    CREATE,
    TABLE,
    DROP,

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
    UnKnown,
};

}

#endif
