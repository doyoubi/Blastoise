#include "buffer.h"
#include "utils/assert.h"

namespace blt
{

PagePool::PagePool(size_t pageSum, InitPageFunc initFunc, FlushPageFunc flushFunc)
    : pageBuffer_(pageSum),
    descNodes_(pageSum),
    head_(&descNodes_.front()),
    tail_(&descNodes_.back()),
    pageSum_(pageSum),
    initPageFunc_(initFunc),
    flushPageFunc_(flushFunc)
{
    DEBUG_CHECK(pageSum > 0);
    for(size_t i = 0; i != pageSum; i++)
    {
        descNodes_[i].next = &descNodes_[i+1];
        descNodes_[(i+1) % pageSum].last = &descNodes_[i];
        descNodes_[i].page = &pageBuffer_[i];
    }
}

byte * PagePool::getPageData(int fd, size_t pageNum)
{
    PageKey k = this->hash(fd, pageNum);
    if(pageHash_.count(k))
    {
        this->nodeToHead(pageHash_.at(k));
        return head_->page->data;
    }

    PageDescNode & desc = *tail_;
    if(desc.pinCount > 0)
        return nullptr;
    if(desc.dirty)
        this->flushPageFunc_(desc.fd, desc.pageNum, desc.page->data);

    PageKey old_key = this->hash(desc.fd, desc.pageNum);
    if(pageHash_.count(old_key))
        this->removeHash(old_key);

    this->addHash(k, tail_);
    this->nodeToHead(tail_);
    desc.fd = fd;
    desc.pageNum = pageNum;
    desc.pinCount = 0;
    desc.dirty = false;
    this->initPageFunc_(fd, pageNum, desc.page->data);
    return head_->page->data;
}

void PagePool::markDirty(int fd, size_t pageNum)
{
    PageKey k = this->hash(fd, pageNum);
    DEBUG_CHECK(pageHash_.count(k));
    pageHash_.at(k)->dirty = true;
}

void PagePool::pinPage(int fd, size_t pageNum)
{
    PageKey k = hash(fd, pageNum);
    DEBUG_CHECK(pageHash_.count(k));
    auto desc = pageHash_.at(k);
    ++desc->pinCount;
}

void PagePool::unpinPage(int fd, size_t pageNum)
{
    PageKey k = hash(fd, pageNum);
    DEBUG_CHECK(pageHash_.count(k));
    auto desc = pageHash_.at(k);
    DEBUG_CHECK(desc->pinCount > 0);
    --desc->pinCount;
}

void PagePool::nodeToHead(PageDescNode * n)
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
    STATIC_ASSERT(sizeof(PagePool::PageKey) == 2 * sizeof(int),
        "PageKey should be 2 times larger than int");
    return (long(fd) << (sizeof(fd) * 8)) + pageNum;
}

void PagePool::removeHash(PageKey k)
{
    DEBUG_CHECK(pageHash_.count(k));
    pageHash_.erase(k);
}

void PagePool::addHash(PageKey k, PageDescNode * n)
{
    DEBUG_CHECK(pageHash_.count(k) == 0);
    pageHash_.insert({k, n});
}

}
