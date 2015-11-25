#ifndef BLT_MOCK_H
#define BLT_MOCK_H

#include <memory>

namespace blt
{

// different MockFunc objects generated from the same root MockFunc object
// by copy constructor or assignment share the same `called` state
template<class ReturnType, class... Args>
class MockFunc
{
public:
    MockFunc() : called(new bool(false)) {}

    ReturnType operator()(Args...)
    {
        *called = true;
    }

    bool wasCalled()
    {
        bool temp = *called;
        *called = false;
        return temp;
    }
private:
    std::shared_ptr<bool> called;
};

}

#endif
