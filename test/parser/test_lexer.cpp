#include <string>
#include <gtest/gtest.h>
#define private public
#include "parser/lexer.h"

using namespace blt;
using std::string;

namespace blt
{
extern void toLower(std::string & str);

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
    toLower(str);
    ASSERT_EQ(str, string("aazz09_#"));
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

