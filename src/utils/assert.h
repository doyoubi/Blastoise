#ifndef BLT_ASSERT_H
#define BLT_ASSERT_H

#include <iostream>
#include <string>

namespace blt
{

using std::endl;
using std::cout;
using std::cerr;
using std::string;

template<class MSG>
void DebugCheck(
    bool checkedExpression,
    const char * filename,
    int line,
    MSG errorMsg)
{
    if (checkedExpression) return;
    cerr << filename << " : " << line << endl
        << errorMsg << endl;
}

#define DEBUGCHECK(checkedExpression) DebugCheck(checkedExpression, __FILE__, __LINE__, "")
#define DEBUGCHECK_WITH_MSG(checkedExpression, errorMsg) DebugCheck(checkedExpression, __FILE__, __LINE__, errorMsg)
#define CHECKNULL(ptr) DEBUGCHECK(ptr != nullptr, "null pointer")
#define ERRORMSG(errorMsg) DEBUGCHECK(false, errorMsg)


inline void Check(bool checkedExpression, string errorMsg)
{
    if(checkedExpression) return;
    cerr << errorMsg << endl;
}

#define CHECK(checkedExpression, errorMsg) Check(checkedExpression, errorMsg)

#define STATIC_ASSERT static_assert

}

#endif
