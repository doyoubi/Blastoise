use std::ptr::{read, write};
use libc::malloc;
use ::store::file::{TableFile, FilePage, BitMap, PageHeader};
use ::store::buffer::DataPtr;


#[test]
fn test_page_header() {
    {
        let data;
        unsafe{
            data = malloc(2);
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
            data = malloc(2);
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
fn test_bitmap()
{
    let data = unsafe{ malloc(3) };
    let mut bitmap = BitMap{
        data : data,
        slot_sum : 24,
    };
    bitmap.clean();
    for i in 0..24 {
        assert!(!bitmap.is_inuse(i));
    }
    bitmap.set_inuse(8, true);
    assert!(!bitmap.is_inuse(7));
    assert!(bitmap.is_inuse(8));
    assert!(!bitmap.is_inuse(9));
    bitmap.set_inuse(8 + 7, true);
    assert!(!bitmap.is_inuse(8 + 6));
    assert!(bitmap.is_inuse(8 + 7));
    assert!(!bitmap.is_inuse(8 + 8));
    bitmap.set_inuse(8 + 3, true);
    assert!(!bitmap.is_inuse(8 + 2));
    assert!(bitmap.is_inuse(8 + 3));
    assert!(!bitmap.is_inuse(8 + 4));
    unsafe{
        assert_eq!(read::<u8>(data as *const u8), 0);
        assert_eq!(read::<u8>((data as *const u8).offset(1)), 137);
        assert_eq!(read::<u8>((data as *const u8).offset(2)), 0);
    }
}

#[test]
fn test_file_page_data_offset() {

}
