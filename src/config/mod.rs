use std::collections::HashMap;
use std::fs;
use tklog::info;

#[derive(Debug)]
pub struct Config{
    pub dockerfile_dir: String,
    pub use_created_image: bool
}

pub fn read_config() -> Config{
    let config = fs::read_to_string("config.txt").unwrap();
    let map = read_to_hashmap(config);
    Config{
        dockerfile_dir: String::from(&map["dockerfile_dir"]),
        use_created_image: map["use_created_image"].parse::<bool>().unwrap(),
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
            info!(format!("读取到配置：{}={}", &key, &val));
            map.insert(key, val);
        });
    map
}

#[test]
fn test_read_config(){
    let config = read_config();
    println!("{:?}",config);
}