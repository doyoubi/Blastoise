use std::io::Write;
use bytes::Buf;
use ::server::buf::Buffer;


#[test]
fn test_buf() {
    {
        let mut buf = Buffer::new(8);
        check_ok!(buf.write(b"123456"));
        assert_eq!(buf.bytes(), b"123456");
        assert!(!buf.should_shift(2));
        check_ok!(buf.write(b"789"));
        assert_eq!(buf.bytes(), b"123456789");
    }
    {
        let mut buf = Buffer::new(8);
        check_ok!(buf.write(b"123456"));
        assert_eq!(buf.bytes(), b"123456");
        buf.advance(2);
        assert_eq!(buf.bytes(), b"3456");
        assert!(!buf.should_shift(2));
        check_ok!(buf.write(b"78"));
        assert_eq!(buf.bytes(), b"345678");
    }
    {
        let mut buf = Buffer::new(8);
        check_ok!(buf.write(b"123456"));
        assert_eq!(buf.bytes(), b"123456");
        buf.advance(2);
        assert_eq!(buf.bytes(), b"3456");
        assert!(buf.should_shift(3));
        assert!(buf.should_shift(4));
        check_ok!(buf.write(b"789"));
        assert_eq!(buf.bytes(), b"3456789");
    }
}
