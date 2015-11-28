#include <string>
#include <gtest/gtest.h>
#define private public
#include "parser/lexer.h"

using namespace blt;

namespace blt
{
extern void toLower(std::string & str);
}

TEST(UtilsTest, ToLowerTest)
{
    using namespace std;
    string str("aAzZ09_#");
    toLower(str);
    ASSERT_EQ(str, string("aazz09_#"));
}
