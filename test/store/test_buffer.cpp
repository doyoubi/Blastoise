#include <gtest/gtest.h>
#define private public
#include "store/buffer.h"


using namespace blt;

TEST(HashTest, FdAndPageNum)
{
    PagePool pool(1);
    ASSERT_EQ(pool.hash(1,1), pool.hash(1,1));
    ASSERT_NE(pool.hash(1,1), pool.hash(1,2));
    ASSERT_NE(pool.hash(1,1), pool.hash(2,1));
}

TEST(LRUTest, OnePage)
{
    PagePool pool(1);
    byte * data1 = pool.getPageData(1, 1);
    byte * data2 = pool.getPageData(1, 1);
    ASSERT_EQ(data1, data2);
}

TEST(LRUTest, TwoPage)
{
    PagePool pool(2);
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
