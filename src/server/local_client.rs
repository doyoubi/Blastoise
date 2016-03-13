use std::io::{stdin, stdout};
use std::io::Write;
use ::store::table::TableManager;
use ::store::tuple::TupleValue;
use ::utils::config::Config;
use super::handler::{sql_handler, ResultHandler};


#[derive(Debug)]
pub struct LocalClient;

impl LocalClient {
    pub fn shell_loop(&mut self) {
        let config = Config::from_cwd_config();
        let mut manager = TableManager::make_ref(&config);
        let mut sql = String::new();
        let mut line = String::new();
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
                        println!("processing {:?}", sql);
                        sql.pop();  // remove ';'
                        sql_handler(&sql, self, &mut manager);
                        sql.clear();
                    }
                    line.clear();
                }
                Err(error) => println!("error: {}", error),
            }
        }
    }
}

impl ResultHandler for LocalClient {
    fn handle_error(&mut self, err_msg : String) {
        println!("{}", err_msg);
    }
    fn handle_tuple_data(&mut self, tuple_value : Option<Vec<TupleValue>>) {
        match tuple_value {
            Some(values) => println!("{:?}", values),
            None => println!("end of tuples"),
        }
    }
}
