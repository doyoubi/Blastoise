use std::io::{stdin, stdout};
use std::io::Write;
use std::rc::Rc;
use std::cell::RefCell;
use ::store::table::TableManager;
use ::store::tuple::TupleData;
use ::store::table::AttrType;
use ::store::tuple::gen_tuple_value;
use ::utils::config::Config;
use super::handler::{sql_handler, ResultHandler, process_table_command};


#[derive(Debug)]
pub struct LocalClient;

impl LocalClient {
    pub fn shell_loop(&mut self) {
        let config = Config::from_cwd_config();
        let mut manager = Rc::new(RefCell::new(TableManager::from_json_file(&config)));
        let mut sql = String::new();
        let mut line = String::new();
        let mut process = Process::new();
        loop {
            print!("Blastoise> ");
            stdout().flush().ok();
            match stdin().read_line(&mut line) {
                Ok(n) => {
                    line.pop();  // remove '\n'
                    if n == 0 { continue }
                    if line == "q" { break; }
                    sql.push_str(&line);
                    if let Some(';') = line.chars().rev().take(1).next() {
                        sql.pop();  // remove ';'
                        if let Ok(out) = process_table_command(&sql, &manager) {
                            println!("{}", out);
                        } else {
                            println!("processing {:?}", sql);
                            sql_handler(&sql, &mut process, &mut manager);
                            process = Process::new();
                        }
                        sql.clear();
                    }
                    line.clear();
                }
                Err(error) => println!("error: {}", error),
            }
        }
    }
}

#[derive(Debug)]
pub struct Process {
    attr_desc : Vec<AttrType>,
    attr_index : Vec<usize>,
}

impl Process {
    pub fn new() -> Process {
        Process{
            attr_desc : Vec::new(),
            attr_index : Vec::new(),
        }
    }
}

impl ResultHandler for Process {
    fn handle_error(&mut self, err_msg : String) {
        println!("{}", err_msg);
    }
    fn handle_tuple_data(&mut self, tuple_data : Option<TupleData>) {
        match tuple_data {
            Some(data) => {
                let value = gen_tuple_value(&self.attr_desc, data);
                println!("{:?}", value);
            }
            None => println!("end"),
        }
    }
    fn set_tuple_info(&mut self, attr_desc : Vec<AttrType>, attr_index : Vec<usize>) {
        self.attr_desc = attr_desc;
        self.attr_index = attr_index;
    }
}
