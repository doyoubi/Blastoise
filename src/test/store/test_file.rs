use std::ptr::{read, write};
use libc::malloc;
use ::store::file::{TableFile, FilePage, BitMap, PageHeader};
use ::store::buffer::DataPtr;


#[test]
fn test_page_header() {
    {
        let data;
        unsafe{
            data = malloc(8);
            write::<u32>(data as *mut u32, 233);
            write::<u32>(data.offset(4) as *mut u32, 666);
        }
        let mut header = PageHeader{
            slot_sum : 0,
            first_free_page : 0,
            data : data,
        };
        header.init_from_page_data();
        assert_eq!(header.slot_sum, 233);
        assert_eq!(header.first_free_page, 666);
    }
    {
        let data;
        unsafe{
            data = malloc(8);
            write::<u32>(data as *mut u32, 111);
            write::<u32>(data.offset(4) as *mut u32, 222);
        }
        let mut header = PageHeader{
            slot_sum : 233,
            first_free_page : 666,
            data : data,
        };
        header.save_to_page_data();
        assert_eq!(unsafe{read::<u32>(data as *const u32)}, 233);
        assert_eq!(unsafe{read::<u32>(data.offset(4) as *const u32)}, 666);
    }
}

#[test]
fn test_file_page_data_offset() {

}
