#include "buffer.h"

namespace blt
{

PagePool::PagePool(size_t pageSum)
    : pageBuffer_(pageSum),
    descNodes_(pageSum),
    head_(INVALID_INDEX),
    tail_(INVALID_INDEX),
    pageSum_(pageSum)
{
    DEBUG_CHECK(pageSum > 0);
    for(size_t i = 0; i != pageSize; i++)
    {
        descNodes_[i].next = i + 1;
        descNodes_[(i+1) % pageSize].last = i;
    }
}

byte * PagePool::getPageData(int fd, size_t pageNum)
{
    PageKey k = this->hash(fd, pageNum);
    if(pageHash_.count(k))
    {
        PageIndex i = pageHash_.at(k);
        return pageBuffer_[i].data;
    }

    PageDescNode & desc = descNodes_[tail_];
    if(desc.pinCount > 0)
        return nullptr;
    this->nodeToHead(tail_);
    if(desc.dirty)
        this->flushPage(desc.fd, desc.pageNum);
    desc.fd = fd;
    desc.pageNum = pageNum;
    desc.pinCount = 0;
    desc.dirty = false;
    return pageBuffer_[head_].data;
}

void PagePool::pinPage(int fd, size_t pageNum)
{
    PageKey k = hash(fd, pageNum);
    DEBUG_CHECK(pageHash_.count(k));
    Page & page = pageHash_.at(k);
    page.pinCount++;
}

void PagePool::unpinPage(int fd, size_t pageNum)
{
    PageKey k = hash(fd, pageNum);
    DEBUG_CHECK(pageHash_.count(k));
    Page & page = pageHash_.at(k);
    DEBUG_CHECK(page.pinCount > 0);
    page.pinCount--;
}

void flushPage(int fd, size_t pageNum)
{
}

void PagePool::nodeToHead(PageIndex i)
{
    if(i == head_) return;
    if(i == tail_)
    {
        tail_ = tail_.last;
        head_ = i;
    }
    // remove
    auto last = [](size_t i) { return (i - 1 + pageSum_) % pageSum_; }
    auto next = [](size_t i) { return (i + 1) % pageSum_; }
    PageIndex currLast = last(i);
    PageIndex currNext = next(i);
    descNodes_[currLast].next = currNext;
    descNodes_[currNext].last = currLast;
    // add to head
    descNodes_[i].last = tail_;
    descNodes_[i].next = head_;
    descNodes_[tail_].next = i;
    descNodes_[head_].last = i;
    head_ = i;
}

PagePool::PageKey PagePool::hash(int fd, size_t pageNum)
{
    STATIC_ASSERT(sizeof(PagePool::PageKey) == 2 * sizeof(int));
    return (fd << (sizeof(fd) * 8)) + pageNum;
}

}