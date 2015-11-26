#ifndef BLT_FILE_H
#define BLT_FILE_H

#include "utils/type.h"

namespace blt
{

class PagePool;

class PageHandle
{
public:
    PageHandle(PagePool & pool, int fd, size_t pageNum);
    ~PageHandle();
    void pin();
    void unpin();
    byte * getData();
    size_t getPageNum() { return pageNum_; }
private:
    PagePool & pool_;
    int fd_;
    size_t pageNum_;
    bool pinned_;
};


class FileBuffer
{
public:
    FileBuffer(PagePool & pool, int fd);
    PageHandle getPageHandle(int pageNum);
private:
    PagePool & pool_;
    int fd_;
};


class File
{
public:
};


class FileBufferManager
{
public:
    FileBufferManager();
};

}

#endif
