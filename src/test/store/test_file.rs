use std::ptr::{read, write};
use std::rc::Rc;
use std::cell::Cell;
use std::sync::{Arc, RwLock};
use libc::malloc;
use ::store::file::{TableFile, FilePage, BitMap, PageHeader};
use ::store::buffer::{DataPtr, Page};
use ::store::table::{Table, Attr, AttrType};
use ::parser::common::{ValueExpr, ValueType};
use ::test::store::test_buffer::MockCacheSaver;


#[test]
fn test_page_header() {
    {
        let data;
        unsafe{
            data = malloc(8);
            write::<u32>(data as *mut u32, 233);
            write::<u32>((data as *mut u32).offset(1), 666);
        }
        let mut header = PageHeader{
            slot_sum : 233,
            first_free_slot : 0,
            data : data,
        };
        header.init_from_page_data();
        assert_eq!(header.slot_sum, 233);
        assert_eq!(header.first_free_slot, 666);
    }
    {
        let data;
        unsafe{
            data = malloc(8);
            write::<u32>(data as *mut u32, 111);
            write::<u32>((data as *mut u32).offset(1), 222);
        }
        let mut header = PageHeader{
            slot_sum : 233,
            first_free_slot : 666,
            data : data,
        };
        header.save_to_page_data();
        assert_eq!(unsafe{read::<u32>(data as *const u32)}, 233);
        assert_eq!(unsafe{read::<u32>((data as *const u32).offset(1))}, 666);
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
    assert_eq!(bitmap.get_first_free_slot(), 0);
    bitmap.set_inuse(0, true);
    assert_eq!(bitmap.get_first_free_slot(), 1);
    for i in 1..8 {
        bitmap.set_inuse(i, true);
    }
    assert_eq!(bitmap.get_first_free_slot(), 8 + 1);
}

fn gen_test_table() -> Table {
    Table{
        name : "message".to_string(),
        attr_list : vec![
            Attr{
                name : "id".to_string(),
                attr_type : AttrType::Int,
                primary : true,
                nullable : false,
            },
            Attr{
                name : "content".to_string(),
                attr_type : AttrType::Char{ len : 6 },
                primary : false,
                nullable : false,
            },
            Attr{
                name : "score".to_string(),
                attr_type : AttrType::Float,
                primary : false,
                nullable : true,
            },
        ],
    }
}

#[test]
fn test_file_page_insert() {
    // tuple: int, char(6), float
    let saver = Box::new(Rc::new(Cell::new(MockCacheSaver{
        fd : 0,  // not used
        page_index: 0,  // not used
        saved : false,  // not used
    })));
    let table = gen_test_table();
    let tuple_desc = table.gen_tuple_desc();
    let mem_page = Arc::new(RwLock::new(Page{
        fd : 1,
        page_index : 2,
        data : unsafe{ malloc(tuple_desc.tuple_len) },
        dirty : false,
        saver : saver,
    }));
    let mut file_page = FilePage::new(mem_page, tuple_desc.tuple_len);
    let mut value_list = vec![
        ValueExpr{ value : "233".to_string(), value_type : ValueType::Integer },
        ValueExpr{ value : "abcdef".to_string(), value_type : ValueType::String },
        ValueExpr{ value : "666.666".to_string(), value_type : ValueType::Float },
    ];
    assert_eq!(file_page.header.first_free_slot, 0);
    assert_eq!(file_page.is_inuse(0), false);
    file_page.insert(&value_list, &tuple_desc);
    assert_eq!(file_page.header.first_free_slot, 1);
    assert_eq!(file_page.is_inuse(0), true);
    value_list[0].value = "777".to_string();
    value_list[1].value = "dyb".to_string();
    value_list[2].value = "12345.777".to_string();
    file_page.insert(&value_list, &tuple_desc);
    assert_eq!(file_page.header.first_free_slot, 2);
    assert_eq!(file_page.is_inuse(1), true);
}

#[test]
fn test_file_page_data_offset() {

}
