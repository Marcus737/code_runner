use std::collections::HashMap;
use std::fs;
use log::info;
use anyhow::{ Context, Result};

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

pub fn read_config() -> Result<Config> {
    let config = fs::read_to_string("config.txt")?;
    let map = read_to_hashmap(config);
    

    Ok(Config{
        dockerfile_dir: String::from(get_val_in_map(&map, "dockerfile_dir")?),
        use_created_image: get_val_in_map(&map, "use_created_image")?
                                    .parse::<bool>()?,
        container_prefix: String::from(get_val_in_map(&map, "container_prefix")?),
        src_code_filename: String::from(get_val_in_map(&map, "src_code_filename")?),
        input_filename: String::from(get_val_in_map(&map, "input_filename")?),
        run_result_filename: String::from(get_val_in_map(&map, "run_result_filename")?),
        repository_name:String::from(get_val_in_map(&map, "repository_name")?),
    })
}

fn get_val_in_map<'a>(map: &'a HashMap<String, String>, key: &str) -> Result<&'a String> {
    Ok(map.get(key)
        .with_context(||{
            format!("cannot find {} in config.txt", key)
        })?)
}


fn read_to_hashmap(config: String) -> HashMap<String, String>{
    let mut map = HashMap::new();
    config.split("\n")
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