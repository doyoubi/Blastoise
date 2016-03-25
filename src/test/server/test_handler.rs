use ::server::handler::{sql_handler, ResultHandler};
use ::store::tuple::TupleData;
use ::store::table::{TableManager, AttrType};
use ::utils::config::Config;


#[derive(Debug)]
struct MockHandler {
    pub helper_data : String,
}

impl MockHandler {
    pub fn new() -> MockHandler {
        // Box::new(MockHandler{ helper_data : String::new() })
        MockHandler{ helper_data : String::new() }
    }
}

impl ResultHandler for MockHandler {
    fn handle_error(&mut self, err_msg : String) {
        self.helper_data = err_msg
    }
    fn handle_tuple_data(&mut self, tuple_data : Option<TupleData>) {
        match tuple_data {
            Some(..) => self.helper_data.push('1'),
            None => self.helper_data.push('0'),
        }
    }
    fn set_tuple_info(&mut self, _attr_desc : Vec<AttrType>, _attr_index : Vec<usize>) {}
    fn handle_non_query_finished(&mut self) {}
}


#[test]
fn test_handler() {
    let config = Config::new(&r#"
        max_memory_pool_page_num = 2
        table_meta_dir = "test_file/table_meta/"
        table_file_dir = "test_file/table_file""#.to_string());
    let manager = TableManager::make_ref(&config);
    let mut handler = MockHandler::new();
    let sql = "create table msg(id int not null primary)".to_string();
    sql_handler(&sql, &mut handler, &manager);
    assert_eq!(handler.helper_data, "");
}
