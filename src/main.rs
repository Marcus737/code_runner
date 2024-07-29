use std::collections::HashMap;
use std::fs;

use crate::config::read_config;
use crate::image::{build_images, Image};
use crate::tape::{Tape, TapePlayer};
use crate::tape::java_tape::JavaTape;

mod util;
mod tape;
mod config;
mod image;

#[derive(Debug)]
struct Code{
    id: String,
    language_id: String,
    input: String,
    code_string: String,
    time_limit: u64,
    memory_limit: u64,
    image_id: String
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
            image_id: String::from(&self.image_id),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.id = String::from(&source.id);
        self.language_id = String::from(&source.language_id);
        self.input = String::from(&source.input);
        self.memory_limit = source.memory_limit;
        self.code_string = String::from(&source.code_string);
        self.time_limit = source.time_limit;
        self.image_id =  String::from(&source.image_id);
    }
}

fn main() {
    let config = read_config();

    let map = build_images(config);

    let mut code = Code{
        id: "111".parse().unwrap(),
        language_id: String::from("java8"),
        input: String::from("1 33\n"),
        code_string: fs::read_to_string("Main.java").unwrap(),
        time_limit: 30000,
        memory_limit: 16,
        image_id: String::new()
    };


    let tape: Box<dyn Tape> = get_tape(&mut code, &map).unwrap();


    let mut player = TapePlayer::new(
        tape
    );

    player.play(code);


}

fn get_tape(code: &mut Code, map: &HashMap<String, Image>) -> Option<Box<dyn Tape>>
{
    let language_id = &code.language_id;
    code.image_id = String::from(&map.get(language_id).unwrap().image_id);
    if language_id.starts_with("java") {
        Some(
            Box::new(JavaTape::new())
        )
    }else {
        None
    }
}



