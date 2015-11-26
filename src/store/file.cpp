#include "file.h"
#include "buffer.h"

namespace blt
{

// PageHandle
PageHandle::PageHandle(PagePool & pool, int fd, size_t pageNum)
    : pool_(pool),
    fd_(fd),
    pageNum_(pageNum),
    pinned_(false)
{}

PageHandle::~PageHandle()
{
    if(pinned_)
        this->unpin();
}

byte * PageHandle::getData()
{
    return pool_.getPageData(fd_, pageNum_);
}

void PageHandle::pin()
{
    pinned_ = true;
    pool_.pinPage(fd_, pageNum_);
}

void PageHandle::unpin()
{
    pinned_ = false;
    pool_.unpinPage(fd_, pageNum_);
}


// FileBuffer
FileBuffer::FileBuffer(PagePool & pool, int fd)
    : pool_(pool),
    fd_(fd)
{}

PageHandle FileBuffer::getPageHandle(int pageNum)
{
    // TODO: add pageNum check
    return PageHandle(pool_, fd_, pageNum);
}

}
