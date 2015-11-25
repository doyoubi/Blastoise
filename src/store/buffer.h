#ifndef BST_BUFFER_H
#define BST_BUFFER_H

#include "utils/type.h"
#include "utils/consts.h"

#include <vector>
#include <unordered_map>
#include <functional>


namespace blt
{

struct Page
{
    static const size_t PAGE_SIZE = (1 << 12) * sizeof(byte);  // 4kb
    byte data[PAGE_SIZE];
};


struct PageDescNode
{
    Page * page;
    int fd;
    size_t pageNum; // page number of file
    size_t pinCount;
    bool dirty;
    PageDescNode * last;
    PageDescNode * next;

    PageDescNode()
        : page(nullptr),
        fd(INVALID_FD),
        pageNum(0),
        pinCount(0),
        dirty(false),
        last(nullptr),
        next(nullptr)
    {}
};


class PagePool
{
public:
    typedef long PageKey;
    typedef std::function<void(int fd, size_t pageNum, byte * data)> InitPageFunc;
    typedef std::function<void(int fd, size_t pageNum, byte * data)> FlushPageFunc;

    PagePool(size_t pageSum, InitPageFunc initFunc, FlushPageFunc flushFunc);
    byte * getPageData(int fd, size_t pageNum);
    void markDirty(int fd, size_t pageNum);

    void pinPage(int fd, size_t pageNum);
    void unpinPage(int fd, size_t pageNum);
private:
    PageKey hash(int fd, size_t pageNum);
    void removeHash(PageKey k);
    void addHash(PageKey k, PageDescNode * n);
    void nodeToHead(PageDescNode * n);

    std::vector<Page> pageBuffer_;
    std::vector<PageDescNode> descNodes_;  // circular linked list
    std::unordered_map<PageKey, PageDescNode*> pageHash_;

    PageDescNode * head_;
    PageDescNode * tail_;
    size_t pageSum_;
    InitPageFunc flushPageFunc_;
    FlushPageFunc initPageFunc_;
};

}


#endif
