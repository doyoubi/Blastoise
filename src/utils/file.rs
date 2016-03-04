use std::fs::{metadata, create_dir};
use std::path::Path;


pub fn path_join(path : &String, file : &String) -> String {
    match metadata(path) {
        Ok(m) => assert!(m.is_dir()),
        Err(e) => panic!("path_join fail {:?}, {:?}", path, e),
    }
    Path::new(path).join(file).to_str().unwrap().to_string()
}

pub fn ensure_dir_exist(path : &String) {
    match metadata(path) {
        Ok(m) => {
            if !m.is_dir() {
                panic!("{:?} is not a directory", path);
            }
        }
        Err(..) => {
            // directory not exist or permission denied
            check_ok!(create_dir(path));
        }
    }
}
