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


struct PageHeader
{
    size_t slotSum;
    int firstFreePage;
};


class Bitmap
{
public:
private:
    size_t size_;
    byte * data_;
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


struct FileHeader
{
    int firstFreePage;
    size_t pageSum;
};


class Record
{
public:
private:
    byte * data_;
};


class File
{
public:
    File(const char * filename);
private:
    FileHeader header_;
    int fd_;
};


class FileBufferManager
{
public:
    FileBufferManager();
};

}

#endif
