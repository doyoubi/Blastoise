#include <sstream>
#include <algorithm>

#include "lexer.h"

namespace blt
{

using std::string;

void toLower(string & str)
{
    auto isUpperLetter = [](char c){
        return 'A' <= c && c <= 'Z';
    };
    for(char & c : str)
        if(isUpperLetter(c))
            c = c + ('a' - 'A');
}

TokenType strToTokenType(const string & str)
{
    return str == "select" ? TokenType::Select :
        str == "from" ? : TokenType::From :
        str == "where" ? : TokenType::Where :
        str == "order" ? : TokenType::Order :
        str == "by" ? : TokenType::By :
        str == "group" ? : TokenType::Group :
        str == "having" ? : TokenType::Having :
        str == "insert" ? : TokenType::Insert :
        str == "values" ? : TokenType::Values :
        str == "update" ? : TokenType::Update :
        str == "set" ? : TokenType::Set :
        str == "delete" ? : TokenType::Delete :
        str == "create" ? : TokenType::CREATE :
        str == "table" ? : TokenType::TABLE :
        str == "drop" ? : TokenType::DROP :
        str == "null" ? : TokenType::Null :
        str == "and" ? : TokenType::And :
        str == "or" ? : TokenType::Or :
        str == "not" ? : TokenType::Not :
        str == "is" ? : TokenType::Is :
        TokenType::UnKnown;
}

bool TokenLine::checkUnEscapeString(const string & s, Token::Ptr & token)
{
    std::stringstream ss;
    bool escaping = false;
    for (size_t i = 0; i <= s.size(); i++)
    {
        char c = i == s.size() ? '\0' : s[i];
        if (escaping)
        {
            if (c == 'a')
                ss << '\a';
            else if (c == 'b')
                ss << '\b';
            else if (c == 'f')
                ss << '\f';
            else if (c == 'n')
                ss << '\n';
            else if (c == 'r')
                ss << '\r';
            else if (c == 't')
                ss << '\t';
            else if (c == 'v')
                ss << '\v';
            else if (c == '\\')
                ss << '\\';
            else if (c == '\'')
                ss << '\'';
            else if (c == '"')
                ss << '\"';
            else if (c == '0')
                ss << '\0';
            else
            {
                CompileError error = {
                    CompileErrorType::Lexer_InvalidEscapeChar,
                    token,
                    "invalid escape char found in string literal"
                };
                errors.push_back(error);
                return false;
            }
            escaping = false;
        }
        else
        {
            if (c == '\\')
                escaping = true;
            else if (i != s.size())
                ss << c;
        }
    }
    token->value = ss.str();
    return true;
}

TokenType convertSingleCharToken(char c)
{
    return c == '(' ? : TokenType::OpenBracket :
        c == ')' ? TokenType::CloseBracket :
        c == ',' ? TokenType::Comma :
        c == '+' ? TokenType::Add :
        c == '-' ? TokenType::Sub :
        c == '*' ? TokenType::Star :
        c == '/' ? TokenType::Div :
        c == '%' ? TokenType::Mod :
        c == '<' ? TokenType::LT :
        c == '>' ? TokenType::GT :
        c == '=' ? TokenType::EQ :
        c == '.' ? TokenType::GetMember :
        TokenType::UnKnown;
}

TokenType convertTwoCharToken(char curr, char next)
{
    return curr == '!' && next == '=' ? TokenType::NE :
        curr == '<' && next == '=' ? TokenType::LE :
        curr == '>' && next == '=' ? TokenType::GE :
        TokenType::UnKnown;
}

bool isIgnoreChar(char c)
{
    char ignored[] = {'\n', '\0', '\t', '\r', ' '};
    return end(ignored) != std::find(begin(ignored), end(ignored), c);
}

TokenLine::Ptr TokenLine::parse(const string & codeString)
{
    auto line = std::make_shared<TokenLine>();

    enum class State
    {
        Begin,
        InInteger,
        InFloat,
        InString,
        InStringEscaping,
        InIdentifier,
    };

    State state = State::Begin;
    auto head = std::cbegin(codeString);
    auto tail = std::cend(codeString);
    auto headUnusedTag = tail;
    auto charIt = std::begin(codeString);

    auto addToken = [&](const string value, TokenType type){
        auto tokenHead = head == headUnusedTag ? charIt : head;
        auto token = std::make_shared<CodeToken>(
            distance(std::begin(codeString), tokenHead) + 1, value, type
        );
        if(type == TokenType::Identifier)
        {
            TokenType t = strToTokenType(value);
            if(t != TokenType::UnKnown)
                token->type = t;
        }
        else if (type == CodeTokenType::StringLiteral)
        {
            bool success = line->UnEscapeString(value, token);
            if (!success)
            {
            } // treat it as an valid string, but without escape, raise an error and go on.
        }
        line->tokens.push_back(token);
    };

    auto addError = [&](CompileErrorType errorType, const string value, const string errorMsg){
        auto tokenHead = head == headUnusedTag ? charIt : head;
        auto token = std::make_shared<CodeToken>(
            distance(std::begin(codeString), tokenHead) + 1, value, CodeTokenType::UnKnown
            );
        CompileError error = {
            errorType, token, errorMsg
        };
        line->errors.push_back(error);
    };

    while(true)
    {
        char c = charIt == tail ? '\0' : *charIt;
        char nextChar = (charIt != tail && std::next(charIt) != tail) ? *std::next(charIt) : '\0';
        switch()
        {
        case State::Begin:
            if(isIgnoreChar(c))
                break;
            else if(TokenType::UnKnown != convertTwoCharToken(c, nextChar))
                addToken({c, nextChar}, convertTwoCharToken(c, nextChar))
            else if(TokenType::UnKnown != convertSingleCharToken(c))
                addToken(string(1, c), convertSingleCharToken(c));
            else if(c == '"')
            {
                state = State::InString;
                head = charIt;
            }
            else if('0' <= c && c <= '9')
            {
                state = State:InInteger;
                head = charIt;
            }
            else if ('a' <= c && c <= 'z' || 'A' <= c && c <= 'Z' || c == '_')
            {
                state = State::InIdentifier;
                head = charIt;
            }
            else
            {
                addError(CompileErrorType::Lexer_UnexpectedChar, string(1, c),
                    "illegal char found: '" + string(1, c) + "'");
            }
            break;
        case State::InIdentifier:
            if ('a' <= c && c <= 'z' || 'A' <= c && c <= 'Z' || c == '_' || '0' <= c && c <= '9')
                {} // go on
            else
            {
                addToken(string(head, charIt), TokenType::Identifier);
                head = headUnusedTag;
                state = State::Begin;
                --charIt;  // let next loop handle separator
            }
            break;
        case State::InString:
            if (c == '\n' || c == '\0')
            {
                addError(CompileErrorType::Lexer_InCompleteString, string(head, charIt),
                    "incomplete string, string must be closed with '\"'");
                head = headUnusedTag;
                state = State::Begin;
                --charIt;  // let next loop handle separator
            }
            else if (c == '\\')
            {
                state = State::InStringEscaping;
            }
            else if (c == '"')
            {
                addToken(string(std::next(head), charIt), CodeTokenType::StringLiteral);
                head = headUnusedTag;
                state = State::Begin;
            }
            else {} // go on
            break;
        case State::InStringEscaping:
                if (c == '\n' || c == '\0')
                {
                    addError(CompileErrorType::Lexer_InCompleteString, string(head, charIt),
                        "incomplete string, string must be closed with '\"'");
                    head = headUnusedTag;
                    state = State::Begin;
                    --charIt;  // let next loop handle separator
                }
                else state = State::InString; // not handle escape char here, just let it there
                break;
        case State::InInteger:
            if ('0' <= c && c <= '9')
                {} // go on
            else if (c == '.')
            {
                if ('0' <= nextChar && nextChar <= '9')
                    state = State::InFloat;
                else
                {
                    // ignore this '.' and treat this token as Float, but raise error
                    addToken(string(head, charIt), CodeTokenType::FloatLiteral);
                    addError(CompileErrorType::Lexer_InvalidFloat, string(head, std::next(charIt)),
                        "'.' should be followed by digit");
                    state = State::Begin;
                    head = headUnusedTag;
                }
            }
            else
            {
                addToken(string(head, charIt), CodeTokenType::IntegerLiteral);
                state = State::Begin;
                head = headUnusedTag;
                --charIt;
                // decrease because the current char is not belong to this token
                // and charIt will increase at the end of loop
            }
            break;
        case State::InFloat:
            if ('0' <= c && c <= '9')
                {} // go on
            else
            {
                addToken(string(head, charIt), CodeTokenType::FloatLiteral);
                state = State::Begin;
                head = headUnusedTag;
                --charIt;
                // decrease because the current char is not belong to this token
                // and charIt will increase at the end of loop
            }
        } // end of switch
        if(charIt == tail) break;
        ++charIt;
    } // end of while

    return line;
}

}
