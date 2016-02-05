use std::option::Option::{Some, None};
use std::result::Result::Ok;
use ::parser::common::{
    check_parse_to_end,
    align_iter,
};


#[test]
fn test_check_parse_to_end() {
    {
        let tokens = gen_token!("");
        assert_eq!(tokens.len(), 0);
        let it = tokens.iter();
        let res = check_parse_to_end(&it);
        assert_pattern!(res, None);
    }
    {
        let tokens = gen_token!("un_parsed_token");
        assert_eq!(tokens.len(), 1);
        let it = tokens.iter();
        let res = check_parse_to_end(&it);
        assert_pattern!(res, Some(_));
    }
}

#[test]
fn test_align_iter() {
    let tokens = gen_token!("1 2 3");
    assert_eq!(tokens.len(), 3);
    let mut i = tokens.iter();
    let mut j = tokens.iter();
    j.next();
    assert_eq!(i.len(), 3);
    assert_eq!(j.len(), 2);
    align_iter(&mut i, &mut j);
    assert_eq!(i.len(), j.len())
}
