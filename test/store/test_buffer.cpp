#include <gtest/gtest.h>
#define private public
#include "store/buffer.h"


using namespace blt;

TEST(LRUTEST, OnePage)
{
    // test for crash
    PagePool pool(1);
    byte * data = pool.getPageData(1, 1);
}

TEST(LRUTEST, TwoPage)
{
    PagePool pool(1);
    byte * data1 = pool.getPageData(1, 1);
    data1[0] = 'a';
    ASSERT_EQ(pool.head_->page->data, data1);
    byte * data2 = pool.getPageData(1, 2);
    data2[0] = 'b';
    ASSERT_NE(data1[0], data2[0]);
    ASSERT_EQ(pool.head_->page->data, data2);
}
