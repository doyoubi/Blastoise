use std::ptr::write;
use std::ptr::read;
use libc::malloc;
use ::parser::condition::ArithExpr;
use ::parser::common::ValueType;
use ::exec::evaluate::{
    PtrMap,
    eval_arith,
};
use ::utils::pointer::pointer_offset;


#[test]
fn test_eval_arith() {
    {
        let arith = gen_parse_result!(ArithExpr::parse, "1 + 2 * 3 - (-6)");
        assert_eq!(eval_arith(&arith, &PtrMap::new()), 13.0);
    }
    {
        let int_p = unsafe{ malloc(8) };
        let float_p = pointer_offset(int_p, 4);
        unsafe{
            write::<i32>(int_p as *mut i32, 233);
            write::<f32>(float_p as *mut f32, 666.666);
        }
        let mut ptr_map = PtrMap::new();
        ptr_map.insert(("student".to_string(), "score".to_string()), (int_p, ValueType::Integer));
        ptr_map.insert(("teacher".to_string(), "score".to_string()), (float_p, ValueType::Float));
        let arith = gen_parse_result!(ArithExpr::parse, "100 + teacher.score + student.score)");
        assert_eq!(eval_arith(&arith, &ptr_map), 999.666);
    }
}