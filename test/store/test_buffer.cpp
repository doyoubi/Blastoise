#include <cstring>
#include <gtest/gtest.h>
#include "../mock.h"
#define private public
#include "store/buffer.h"


using namespace blt;

TEST(HashTest, FdAndPageNum)
{
    auto dummy_func = [](int,size_t,byte*){};
    PagePool pool(1, dummy_func, dummy_func);
    ASSERT_EQ(pool.hash(1,1), pool.hash(1,1));
    ASSERT_NE(pool.hash(1,1), pool.hash(1,2));
    ASSERT_NE(pool.hash(1,1), pool.hash(2,1));
}

TEST(LRUTest, OnePage)
{
    auto dummy_func = [](int,size_t,byte*){};
    PagePool pool(1, dummy_func, dummy_func);
    byte * data1 = pool.getPageData(1, 1);
    byte * data2 = pool.getPageData(1, 1);
    ASSERT_EQ(data1, data2);
}

TEST(LRUTest, TwoPage)
{
    auto dummy_func = [](int,size_t,byte*){};
    PagePool pool(2, dummy_func, dummy_func);
    ASSERT_NE(pool.head_, pool.tail_);

    byte * data1 = pool.getPageData(1, 1);
    data1[0] = 'a';
    ASSERT_EQ(pool.head_->page->data, data1);

    byte * data2 = pool.getPageData(1, 2);
    data2[0] = 'b';
    ASSERT_NE(data1[0], data2[0]);
    ASSERT_EQ(pool.head_->page->data, data2);

    data1 = pool.getPageData(1, 1);
    ASSERT_EQ(pool.head_->page->data, data1);
    ASSERT_EQ(data1[0], 'a');
}

TEST(PageFuncTest, FuncCalledTest)
{
    MockFunc<void, int, size_t, byte*> initFunc;
    MockFunc<void, int, size_t, byte*> flushFunc;
    PagePool pool(2, initFunc, flushFunc);
    ASSERT_FALSE(initFunc.wasCalled());
    ASSERT_FALSE(flushFunc.wasCalled());
    pool.getPageData(1, 1);
    pool.markDirty(1, 1);
    ASSERT_TRUE(initFunc.wasCalled());
    ASSERT_FALSE(flushFunc.wasCalled());
    pool.getPageData(1, 2);
    pool.markDirty(1, 2);
    ASSERT_TRUE(initFunc.wasCalled());
    ASSERT_FALSE(flushFunc.wasCalled());
    pool.getPageData(1, 1);
    ASSERT_FALSE(initFunc.wasCalled());
    ASSERT_FALSE(flushFunc.wasCalled());
    pool.getPageData(1, 3);
    ASSERT_TRUE(initFunc.wasCalled());
    ASSERT_TRUE(flushFunc.wasCalled());
}

TEST(SwapOutPageTest, RestorePageTest)
{
    char dataInFile[] = "1234567";
    char newData[] = "7654321";
    auto initFunc = [dataInFile](int, size_t, byte * data) -> void {
        std::strcpy(data, dataInFile);
    };
    auto flushFunc = [dataInFile](int, size_t, byte * data) -> void {
        std::strcpy((char*)(dataInFile), data);
    };
    PagePool pool(1, initFunc, flushFunc);
    byte * data = pool.getPageData(1, 1);
    pool.markDirty(1, 1);
    ASSERT_STREQ(data, dataInFile);
    std::strcpy(data, newData);
    pool.getPageData(1, 2);
    ASSERT_STREQ(newData, dataInFile);
}
