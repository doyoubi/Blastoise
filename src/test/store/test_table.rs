use std::collections::HashMap;
use rustc_serialize::json::{encode, decode};
use ::store::table::{Table, Attr, AttrType, TableManager};
use ::test::utils::remove_blanks;
use ::utils::config::Config;


#[test]
fn test_attr_type() {
    {
        let json = extract!(encode(&AttrType::Int), Ok(s), s);
        assert_eq!(json, "{\"type\":\"Int\"}");
        let attr = extract!(decode::<AttrType>(&json), Ok(attr), attr);
        assert_pattern!(attr, AttrType::Int);
    }
    {
        let json = extract!(encode(&AttrType::Float), Ok(s), s);
        assert_eq!(json, "{\"type\":\"Float\"}");
        let attr = extract!(decode::<AttrType>(&json), Ok(attr), attr);
        assert_pattern!(attr, AttrType::Float);
    }
    {
        let json = extract!(encode(&AttrType::Char{len:233}), Ok(s), s);
        assert_eq!(json, "{\"len\":\"233\",\"type\":\"Char\"}");
        let attr = extract!(decode::<AttrType>(&json), Ok(attr), attr);
        assert_pattern!(attr, AttrType::Char{len:233});
    }
}

const JSON_DATA : &'static str = r#"
    {
        "author": {
            "name": "author",
            "attr_list": [
                {
                    "name": "id",
                    "attr_type": { "type": "Int" },
                    "primary": true,
                    "nullable": false
                },
                {
                    "name": "name",
                    "attr_type": { "len": "10", "type": "Char" },
                    "primary": false,
                    "nullable": false
                }
            ]
        },
        "book": {
            "name": "book",
            "attr_list": [
                {
                    "name": "id",
                    "attr_type": { "type": "Int" },
                    "primary": true,
                    "nullable": false
                },
                {
                    "name": "author_id",
                    "attr_type": { "type": "Int" },
                    "primary": true,
                    "nullable": true
                }
            ]
        }
    }
    "#;

#[test]
fn test_json_translate() {
    let t1 = Table{
        name : "author".to_string(),
        attr_list : vec![
            Attr{
                name : "id".to_string(),
                attr_type : AttrType::Int,
                primary : true,
                nullable : false,
            },
            Attr{
                name : "name".to_string(),
                attr_type : AttrType::Char{ len : 10 },
                primary : false,
                nullable : false,
            }
        ],
    };
    let t2 = Table{
        name : "book".to_string(),
        attr_list : vec![
            Attr{
                name : "id".to_string(),
                attr_type : AttrType::Int,
                primary : true,
                nullable : false,
            },
            Attr{
                name : "author_id".to_string(),
                attr_type : AttrType::Int,
                primary : true,
                nullable : true,
            }
        ]
    };
    let config = Config::new(&r#"
        max_memory_pool_page_num = 5
        table_meta_dir = "test_file/table_meta/"
        table_file_dir = "test_file/table_file""#.to_string());
    let mut manager = TableManager::new(&config);
    manager.add_table(t1);
    manager.add_table(t2);
    assert_eq!(manager.to_json(), remove_blanks(JSON_DATA));

    let gen_manager = TableManager::from_json(&config, &JSON_DATA.to_string(), false);
    assert_eq!(gen_manager.to_json(), remove_blanks(JSON_DATA));
}

#[test]
fn test_get_table() {
    let config = Config::new(&r#"
        max_memory_pool_page_num = 5
        table_meta_dir = "test_file/table_meta/"
        table_file_dir = "test_file/table_file""#.to_string());
    let manager = TableManager::from_json(&config, &JSON_DATA.to_string(), false);
    let table = extract!(manager.get_table("book"), Some(table), table);
    let table = table.borrow();
    assert_eq!(table.name, "book");
    assert_eq!(table.attr_list.len(), 2);
}

#[test]
fn test_gen_table_set() {
    let config = Config::new(&r#"
        max_memory_pool_page_num = 5
        table_meta_dir = "test_file/table_meta/"
        table_file_dir = "test_file/table_file""#.to_string());
    let manager = TableManager::from_json(&config, &JSON_DATA.to_string(), false);
    let mut used_table = Vec::new();
    used_table.push("author".to_string());
    used_table.push("book".to_string());
    let set = manager.gen_table_set(&used_table);
    assert_eq!(set.tables.len(), 2);
}
