#ifndef BLT_COMPILE_ERROR_H
#define BLT_COMPILE_ERROR_H

namespace blt
{

enum class CompileErrorType
{
    Lexer_InvalidEscapeChar,
    Lexer_UnexpectedChar,
    Lexer_InCompleteString,
    Lexer_InvalidFloat
};

}

#endif
