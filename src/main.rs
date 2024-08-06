
use std::env::set_var;
use std::fs;
use log::info;
use crate::config::read_config;
use crate::image::{build_images};


mod util;
mod config;
mod image;
mod runner;

#[derive(Debug)]
struct Code{
    id: String,
    language_id: String,
    input: String,
    code_string: String,
    time_limit: u64,
    memory_limit: u64,
    image_name: String,
}

impl Clone for Code {
    fn clone(&self) -> Self {
        Code{
            id: String::from(&self.id),
            language_id: String::from(&self.language_id),
            input: String::from(&self.input),
            code_string:  String::from(&self.code_string),
            time_limit: self.time_limit,
            memory_limit: self.memory_limit,
            image_name: String::from(&self.image_name),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.id = String::from(&source.id);
        self.language_id = String::from(&source.language_id);
        self.input = String::from(&source.input);
        self.memory_limit = source.memory_limit;
        self.code_string = String::from(&source.code_string);
        self.time_limit = source.time_limit;
        self.image_name =  String::from(&source.image_name);
    }
}

fn main() {
    set_var("RUST_LOG", "debug");
    env_logger::init();

    let config = read_config();

    let map = build_images(&config);

    info!("{:?}", map);

    let java_code = Code{
        id: "111".parse().unwrap(),
        language_id: String::from("java11"),
        input: String::from("1 33\n"),
        code_string: fs::read_to_string("Main.java").unwrap(),
        time_limit: 3000,
        memory_limit: 16000,
        image_name: format!("{}:{}", config.repository_name, "java11")
    };

    let py3_code =Code{
        id: "1112".parse().unwrap(),
        language_id: String::from("python3"),
        input: String::from("1 3356\n"),
        code_string: fs::read_to_string("main.py").unwrap(),
        time_limit: 5,
        memory_limit: 16000,
        image_name: format!("{}:{}", config.repository_name, "python3")
    };

    let rust_code = Code{
        id: "1112".parse().unwrap(),
        language_id: String::from("rust1.78"),
        input: String::from("1 3356\n"),
        code_string: fs::read_to_string("main.rs").unwrap(),
        time_limit: 500,
        memory_limit: 16000,
        image_name: format!("{}:{}", config.repository_name, "rust1.78")
    };

    
    
    let result = runner::run_code(&java_code, &config);
    println!("{:?}", result);


}


