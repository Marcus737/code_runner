use std::collections::HashMap;
use std::fs;
use log::info;

#[derive(Debug)]
pub struct Config{
    pub dockerfile_dir: String,
    pub use_created_image: bool,
    pub container_prefix: String,
    pub src_code_filename:String,
    pub input_filename:String,
    pub run_result_filename:String,
    pub repository_name:String,
}

pub fn read_config() -> Config{
    let config = fs::read_to_string("config.txt").unwrap();
    let map = read_to_hashmap(config);
    Config{
        dockerfile_dir: String::from(&map["dockerfile_dir"]),
        use_created_image: map["use_created_image"].parse::<bool>().unwrap(),
        container_prefix: String::from(&map["container_prefix"]),
        src_code_filename: String::from(&map["src_code_filename"]),
        input_filename: String::from(&map["input_filename"]),
        run_result_filename: String::from(&map["run_result_filename"]),
        repository_name:String::from(&map["repository_name"]),
    }
}



fn read_to_hashmap(config: String) -> HashMap<String, String>{
    let mut map = HashMap::new();
    config.trim().split("\n")
        .for_each(|line|{
            let line = line.trim();
            let item:Vec<&str> = line.split('=').collect();
            let key = String::from(item[0]);
            let val = String::from(item[1]);
            info!("读取到配置：{}={}", &key, &val);
            map.insert(key, val);
        });
    map
}

#[test]
fn test_read_config(){
    let config = read_config();
    println!("{:?}",config);
}