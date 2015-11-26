#include <gtest/gtest.h>
#define private public
#include <store/file.h>
#include <store/buffer.h>


using namespace blt;

TEST(PageHandleTest, GetDataTest)
{
    auto dummy_func = [](int,size_t,byte*){};
    PagePool pool(1, dummy_func, dummy_func);
    PageHandle handle(pool, 1, 1);
    ASSERT_EQ(handle.getData(), pool.getPageData(1, 1));
}
