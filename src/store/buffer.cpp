#include "buffer.h"

namespace blt
{

PagePool::PagePool(size_t pageSum)
    : pageBuffer_(pageSum),
    descNodes_(pageSum),
    head_(&descNodes_.front()),
    tail_(&descNodes_.back()),
    pageSum_(pageSum)
{
    DEBUG_CHECK(pageSum > 0);
    for(size_t i = 0; i != pageSize; i++)
    {
        descNodes_[i].next = &descNodes_[i+1];
        descNodes_[(i+1) % pageSize].last = &descNodes_[i];
        descNodes_[i].page = &pageBuffer_[i];
    }
}

byte * PagePool::getPageData(int fd, size_t pageNum)
{
    PageKey k = this->hash(fd, pageNum);
    if(pageHash_.count(k))
        PageIndex i = pageHash_.at(k)->page.data;

    PageDescNode & desc = *tail_;
    if(desc.pinCount > 0)
        return nullptr;
    this->nodeToHead(tail_);
    if(desc.dirty)
        this->flushPage(desc.fd, desc.pageNum);
    desc.fd = fd;
    desc.pageNum = pageNum;
    desc.pinCount = 0;
    desc.dirty = false;
    return head_->page.data;
}

void PagePool::pinPage(int fd, size_t pageNum)
{
    PageKey k = hash(fd, pageNum);
    DEBUG_CHECK(pageHash_.count(k));
    auto desc = pageHash_.at(k);
    ++desc->pincout;
}

void PagePool::unpinPage(int fd, size_t pageNum)
{
    PageKey k = hash(fd, pageNum);
    DEBUG_CHECK(pageHash_.count(k));
    auto desc = pageHash_.at(k);
    DEBUG_CHECK(desc->pinCount > 0);
    --desc->pinCount;
}

void flushPage(int fd, size_t pageNum)
{
}

void PagePool::nodeToHead(NodePtr n)
{
    if(n == head_) return;
    if(n == tail_)
    {
        head_ = n;
        tail_ = tail_->last;
    }
    // remove
    n->last->next = n->next;
    n->next->last = n->last;
    // add to head
    n->last = tail_;
    n->next = head_;
    tail_->next = n;
    head_->last = n;
    head_ = n;
}

PagePool::PageKey PagePool::hash(int fd, size_t pageNum)
{
    STATIC_ASSERT(sizeof(PagePool::PageKey) == 2 * sizeof(int));
    return (fd << (sizeof(fd) * 8)) + pageNum;
}

}