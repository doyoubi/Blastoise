use std::fs::OpenOptions;
use std::io::Read;
use toml::{Value, Table, Parser};


#[derive(Debug)]
pub struct Config {
    config : Table,
}

impl Config {
    pub fn from_cwd_config() -> Config {
        let cwd = "./db_config.toml".to_string();
        Self::from_file(&cwd)
    }
    pub fn from_file(config_file : &String) -> Config {
        let mut file = check_ok!(OpenOptions::new().read(true).open(config_file));
        let mut buf = String::new();
        check_ok!(file.read_to_string(&mut buf));
        Self::new(&buf)
    }
    pub fn new(config_str : &String) -> Config {
        let mut parser = Parser::new(&config_str);
        let config = match parser.parse() {
            Some(v) => v,
            None => panic!("db config error: {:?}", parser.errors),
        };
        Config{
            config : config,
        }
    }
    pub fn get_int(&self, path : &str) -> i64 {
        extract!(self.config.get(path), Some(&Value::Integer(n)), n)
    }
    pub fn get_str(&self, path : &str) -> String {
        extract!(self.config.get(path), Some(&Value::String(ref s)), s.clone())
    }
}
