use std::ptr::write;
use std::ptr::read;
use libc::malloc;
use ::parser::condition::{ArithExpr, ConditionExpr};
use ::store::table::AttrType;
use ::exec::evaluate::{
    PtrMap,
    eval_arith,
    eval_cond,
};
use ::utils::pointer::{pointer_offset, write_string};


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
        ptr_map.insert(("student".to_string(), "score".to_string()), (int_p, AttrType::Int));
        ptr_map.insert(("teacher".to_string(), "score".to_string()), (float_p, AttrType::Float));
        let arith = gen_parse_result!(ArithExpr::parse, "100 + teacher.score + student.score)");
        assert_eq!(eval_arith(&arith, &ptr_map), 999.666);
    }
}

#[test]
fn test_eval_cond() {
    {
        let cond = gen_parse_result!(ConditionExpr::parse, "not 2 > 1");
        assert_eq!(eval_cond(&cond, &PtrMap::new()), false);
    }
    {
        let cond = gen_parse_result!(ConditionExpr::parse, "2 > 1 and 1 == 2 or 3 > 1 and 2 >= 2");
        assert_eq!(eval_cond(&cond, &PtrMap::new()), true);
    }
    {
        let cond = gen_parse_result!(ConditionExpr::parse, r#" "bb" != "bb" "#);
        assert_eq!(eval_cond(&cond, &PtrMap::new()), false);
    }
    {
        let int_p = unsafe{ malloc(8) };
        let float_p = pointer_offset(int_p, 4);
        unsafe{
            write::<i32>(int_p as *mut i32, 233);
            write::<f32>(float_p as *mut f32, 666.666);
        }
        let mut ptr_map = PtrMap::new();
        ptr_map.insert(("student".to_string(), "score".to_string()), (int_p, AttrType::Int));
        ptr_map.insert(("teacher".to_string(), "score".to_string()), (float_p, AttrType::Float));
        let cond = gen_parse_result!(ConditionExpr::parse,
            "student.score = 233 and 666.666 = teacher.score and teacher.score > student.score");
        assert_eq!(eval_cond(&cond, &ptr_map), true);
    }
    {
        let s = unsafe{ malloc(8) };
        let f = pointer_offset(s, 4);
        unsafe{
            write_string(s, &"aa".to_string(), 4);
            write::<f32>(f as *mut f32, 666.666);
        }
        let mut ptr_map = PtrMap::new();
        ptr_map.insert(("student".to_string(), "name".to_string()), (s, AttrType::Char{len:4}));
        ptr_map.insert(("teacher".to_string(), "score".to_string()), (f, AttrType::Float));
        let cond = gen_parse_result!(ConditionExpr::parse,
            "student.name = \"aa\" and \"aa\" = student.name and 666.666 = teacher.score");
        assert_eq!(eval_cond(&cond, &ptr_map), true);
    }
}
