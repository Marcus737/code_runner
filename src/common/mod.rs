use rocket::serde::{Deserialize, Serialize};

pub mod util;
pub mod docker_command;

#[derive(Debug)]
#[derive(Serialize)]
#[derive(Deserialize)]
pub struct RunResult{
    pub status: String,
    pub use_time: String,
    pub memory: String,
    pub output: String
}


#[derive(Debug)]
#[derive(FromForm)]
pub struct Code{
    pub _id: Option<String>,
    pub language_id: String,
    pub input: String,
    pub code_string: String,
    pub time_limit: u64,
    pub memory_limit: u64,
}
