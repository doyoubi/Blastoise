#include <string>
#include <gtest/gtest.h>
#define private public
#include "parser/lexer.h"

using namespace blt;
using std::string;

namespace blt
{
extern string toLower(const string & str);

bool operator == (const Token & lhs, const Token & rhs)
{
    return lhs.column == rhs.column
    && lhs.value == rhs.value
    && lhs.type == rhs.type;
}
}

#define ASSERT_TOKEN_EQ(token_, column_, value_, type_) \
do { \
    const Token & tk_ = token_; \
    ASSERT_EQ(column_, tk_.column); \
    ASSERT_EQ(value_, tk_.value); \
    ASSERT_EQ(type_, tk_.type); \
}while(0)

#define ASSERT_COMPILE_ERROR(helper_, errorIndex_, type_) \
do { \
    ASSERT_GT(helper_.tokenLine->errors.size(), errorIndex_); \
    ASSERT_EQ(helper_.tokenLine->errors[errorIndex_].errorType, type_); \
}while(0)

struct TokenTestHelper
{
    TokenTestHelper(const string & code)
        : tokenLine(TokenLine::parse(code))
    {
        iter = tokenLine->tokens.begin();
    }

    Token::Ptr nextToken()
    {
        if(iter == tokenLine->tokens.end())
            return nullptr;
        return *iter++;
    }

    CompileError::List & getErrors()
    {
        return tokenLine->errors;
    }

    Token::Iter iter;
    TokenLine::Ptr tokenLine;
};

TEST(LexerTest, ToLowerTest)
{
    string str("aAzZ09_#");
    ASSERT_EQ(toLower(str), string("aazz09_#"));
}

TEST(LexerTest, EmptyStringTest)
{
    auto tokenLine = TokenLine::parse("");
    ASSERT_EQ(tokenLine->tokens.size(), 0);
    ASSERT_EQ(tokenLine->errors.size(), 0);
}

TEST(LexerTest, IntegerToken)
{
    TokenTestHelper h("1 233 6666");
    ASSERT_TOKEN_EQ(*h.nextToken(), 1, "1", TokenType::IntegerLiteral);
    ASSERT_TOKEN_EQ(*h.nextToken(), 3, "233", TokenType::IntegerLiteral);
    ASSERT_TOKEN_EQ(*h.nextToken(), 7, "6666", TokenType::IntegerLiteral);
    ASSERT_EQ(h.nextToken(), nullptr);
    ASSERT_EQ(h.getErrors().size(), 0);
}

TEST(LexerTest, FloatToken)
{
    TokenTestHelper h("1.0 2.333 12.");
    ASSERT_TOKEN_EQ(*h.nextToken(), 1, "1.0", TokenType::FloatLiteral);
    ASSERT_TOKEN_EQ(*h.nextToken(), 5, "2.333", TokenType::FloatLiteral);
    ASSERT_TOKEN_EQ(*h.nextToken(), 11, "12", TokenType::FloatLiteral);
    ASSERT_EQ(h.nextToken(), nullptr);
    ASSERT_EQ(h.getErrors().size(), 1);
    ASSERT_COMPILE_ERROR(h, 0, CompileErrorType::Lexer_InvalidFloat);
}

TEST(LexerTest, InvalidCharTest)
{
    TokenTestHelper h("1$2##3");
    ASSERT_TOKEN_EQ(*h.nextToken(), 1, "1", TokenType::IntegerLiteral);
    ASSERT_TOKEN_EQ(*h.nextToken(), 3, "2", TokenType::IntegerLiteral);
    ASSERT_TOKEN_EQ(*h.nextToken(), 6, "3", TokenType::IntegerLiteral);
    ASSERT_EQ(h.nextToken(), nullptr);
    ASSERT_EQ(h.getErrors().size(), 3);
    ASSERT_COMPILE_ERROR(h, 0, CompileErrorType::Lexer_UnexpectedChar);
    ASSERT_COMPILE_ERROR(h, 1, CompileErrorType::Lexer_UnexpectedChar);
    ASSERT_COMPILE_ERROR(h, 2, CompileErrorType::Lexer_UnexpectedChar);
}

TEST(LexerTest, StringToken)
{
    TokenTestHelper h(
        "\"a\" \"str1\"\"str2\"" // 16 chars
        "\"\\a\\b\\f\\r\\t\\v\\\\ \\' \\\" \"" // 23 chars
        "\"unfinished escape \\j end\"" // 26 chars
        "\"incomplete string"
        );
    ASSERT_TOKEN_EQ(*h.nextToken(), 1, "a", TokenType::StringLiteral);
    ASSERT_TOKEN_EQ(*h.nextToken(), 5, "str1", TokenType::StringLiteral);
    ASSERT_TOKEN_EQ(*h.nextToken(), 11, "str2", TokenType::StringLiteral);

    ASSERT_TOKEN_EQ(*h.nextToken(), 17, "\a\b\f\r\t\v\\ ' \" ", TokenType::StringLiteral);
    ASSERT_TOKEN_EQ(*h.nextToken(), 40, "unfinished escape \\j end", TokenType::StringLiteral);

    ASSERT_EQ(h.nextToken(), nullptr);
    ASSERT_EQ(h.getErrors().size(), 2);
    ASSERT_COMPILE_ERROR(h, 0, CompileErrorType::Lexer_InvalidEscapeChar);
    ASSERT_COMPILE_ERROR(h, 1, CompileErrorType::Lexer_InCompleteString);
}

TEST(LexerTest, IndentifierToken)
{
    TokenTestHelper h("ident ident2 _233");
    ASSERT_TOKEN_EQ(*h.nextToken(), 1, "ident", TokenType::Identifier);
    ASSERT_TOKEN_EQ(*h.nextToken(), 7, "ident2", TokenType::Identifier);
    ASSERT_TOKEN_EQ(*h.nextToken(), 14, "_233", TokenType::Identifier);
    ASSERT_EQ(h.nextToken(), nullptr);
    ASSERT_EQ(h.getErrors().size(), 0);
}

TEST(LexerTest, KeywordToken)
{
    TokenTestHelper h(
        "select fROM Where order by group having "
        "insert values update set delete "
        "create table drop null and or not is"
        );
    ASSERT_TOKEN_EQ(*h.nextToken(), 1, "select", TokenType::Select);
    ASSERT_TOKEN_EQ(*h.nextToken(), 8, "fROM", TokenType::From);
    ASSERT_TOKEN_EQ(*h.nextToken(), 13, "Where", TokenType::Where);
    ASSERT_TOKEN_EQ(*h.nextToken(), 19, "order", TokenType::Order);
    ASSERT_TOKEN_EQ(*h.nextToken(), 25, "by", TokenType::By);
    ASSERT_TOKEN_EQ(*h.nextToken(), 28, "group", TokenType::Group);
    ASSERT_TOKEN_EQ(*h.nextToken(), 34, "having", TokenType::Having);

    ASSERT_TOKEN_EQ(*h.nextToken(), 40 + 1, "insert", TokenType::Insert);
    ASSERT_TOKEN_EQ(*h.nextToken(), 40 + 8, "values", TokenType::Values);
    ASSERT_TOKEN_EQ(*h.nextToken(), 40 + 15, "update", TokenType::Update);
    ASSERT_TOKEN_EQ(*h.nextToken(), 40 + 22, "set", TokenType::Set);
    ASSERT_TOKEN_EQ(*h.nextToken(), 40 + 26, "delete", TokenType::Delete);

    ASSERT_TOKEN_EQ(*h.nextToken(), 40 + 32 + 1, "create", TokenType::Create);
    ASSERT_TOKEN_EQ(*h.nextToken(), 40 + 32 + 8, "table", TokenType::Table);
    ASSERT_TOKEN_EQ(*h.nextToken(), 40 + 32 + 14, "drop", TokenType::Drop);
    ASSERT_TOKEN_EQ(*h.nextToken(), 40 + 32 + 19, "null", TokenType::Null);
    ASSERT_TOKEN_EQ(*h.nextToken(), 40 + 32 + 24, "and", TokenType::And);
    ASSERT_TOKEN_EQ(*h.nextToken(), 40 + 32 + 28, "or", TokenType::Or);
    ASSERT_TOKEN_EQ(*h.nextToken(), 40 + 32 + 31, "not", TokenType::Not);
    ASSERT_TOKEN_EQ(*h.nextToken(), 40 + 32 + 35, "is", TokenType::Is);

    ASSERT_EQ(h.nextToken(), nullptr);
    ASSERT_EQ(h.getErrors().size(), 0);
}

TEST(LexerTest, OperatorToken)
{
    TokenTestHelper h("(),+-*/%<><=>==!=.");
    ASSERT_TOKEN_EQ(*h.nextToken(), 1, "(", TokenType::OpenBracket);
    ASSERT_TOKEN_EQ(*h.nextToken(), 2, ")", TokenType::CloseBracket);
    ASSERT_TOKEN_EQ(*h.nextToken(), 3, ",", TokenType::Comma);
    ASSERT_TOKEN_EQ(*h.nextToken(), 4, "+", TokenType::Add);
    ASSERT_TOKEN_EQ(*h.nextToken(), 5, "-", TokenType::Sub);
    ASSERT_TOKEN_EQ(*h.nextToken(), 6, "*", TokenType::Star);
    ASSERT_TOKEN_EQ(*h.nextToken(), 7, "/", TokenType::Div);
    ASSERT_TOKEN_EQ(*h.nextToken(), 8, "%", TokenType::Mod);
    ASSERT_TOKEN_EQ(*h.nextToken(), 9, "<", TokenType::LT);
    ASSERT_TOKEN_EQ(*h.nextToken(), 10, ">", TokenType::GT);
    ASSERT_TOKEN_EQ(*h.nextToken(), 11, "<=", TokenType::LE);
    ASSERT_TOKEN_EQ(*h.nextToken(), 13, ">=", TokenType::GE);
    ASSERT_TOKEN_EQ(*h.nextToken(), 15, "=", TokenType::EQ);
    ASSERT_TOKEN_EQ(*h.nextToken(), 16, "!=", TokenType::NE);
    ASSERT_TOKEN_EQ(*h.nextToken(), 18, ".", TokenType::GetMember);
    ASSERT_EQ(h.nextToken(), nullptr);
    ASSERT_EQ(h.getErrors().size(), 0);
}

