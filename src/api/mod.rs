use std::path::PathBuf;
use std::process::Command;
use rocket::serde::json::Json;

use rocket::{Request, State};
use rocket::form::Form;
use serde::{Deserialize, Serialize};
use crate::common::util::run_cmd_get_result;
use crate::common::util::zip_util;
use crate::core::config::Config;
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::tokio::fs;
use uuid::Uuid;
use crate::common::{Code, docker_command, RunResult};
use crate::core::runner;

#[derive(Serialize)]
#[derive(Deserialize)]
pub struct RestResponse<T> {
    msg: String,
    data: Option<T>,
    code: i32
}
impl < T> RestResponse<T> {
    pub fn ok(data: T) -> Self{
        RestResponse{
            msg: String::from("succeed"),
            data: Some(data),
            code: 0
        }
    }
    pub fn ok_msg(msg: String) -> Self{
        RestResponse{
            msg,
            data: None,
            code: 0
        }
    }
    pub fn err(msg: String) -> Self{
        RestResponse{
            msg,
            data: None,
            code: -1
        }
    }
}

// 对于自定义错误类型，你可以这样处理  
#[catch(default)]
pub fn default(status: Status, req: &Request) -> Json<RestResponse<()>>{
    Json(RestResponse::err("server error".to_string()))
}

#[get("/languages")]
pub fn languages(config: &State<Config>) -> Json<RestResponse<Vec<String>>>{

    let mut command = Command::new("docker");
    command.arg("image")
        .arg("ls")
        .arg(format!("--filter=reference={}", config.repository_name))
        .arg("--format={{.Tag}}");

    let command_res = run_cmd_get_result(&mut command).unwrap();

    debug!("{}", command_res);

    let res: Vec<String> = command_res
        .split("\n")
        .filter(|s| s.trim().len() > 0)
        .map(|s| String::from(s))
        .collect();

    let ready = RestResponse::ok(res);

    Json(ready)
}


#[derive(FromForm)]
pub struct UploadZip<'a> {
    language_id: String,
    file: TempFile<'a>
}

#[post("/upload", data = "<zip>")]
pub async fn new_language(config: &State<Config>, mut zip: Form<UploadZip<'_>>) -> Json<RestResponse<()>>{
    
    let dir = "/tmp/code_runner";
    let _result = fs::create_dir_all(&dir).await;
    
    let filename = Uuid::new_v4().to_string();
    let path = PathBuf::from(&dir).join(&filename);
    
    let file = &mut zip.file;
    debug!("{:?}", file);
    
    if let Err(e) = file.persist_to(&path).await {
        error!("{}", e);
        return Json(RestResponse::err(format!("cannot save file {}", e.to_string())));
    }

    //解压缩
    match zip_util::extract(path.to_str().unwrap(), dir) {
        Ok(_) => {
        }
        Err(e) => {
            error!("{}", e);
            return Json(RestResponse::err(format!("cannot extract {}", e.to_string())));
        }
    };
    let pwd = PathBuf::from(&dir).join(&zip.language_id);
    //docker build
    let img_id =  match docker_command::build_image(pwd.to_str().unwrap(), &config.repository_name, &zip.language_id) {
        Ok(s) => {
            s
        }
        Err(e) => {
            let msg = e.to_string();
            error!("{}", &msg);
            return Json(RestResponse::err(msg));
        }
    };
    
    //删除文件
    if let Err(e) =  fs::remove_file(path).await {
      return Json(RestResponse::ok_msg(e.to_string()))  
    }
    if let Err(e) = fs::remove_dir_all(pwd).await{
        return Json(RestResponse::ok_msg(e.to_string()))
    }
    
    Json(RestResponse::ok_msg(img_id))
}

#[get("/remove_language/<language_id>")]
pub fn remove_language(config: &State<Config>, language_id: &str) -> Json<RestResponse<()>> {
    if let Err(e) =  docker_command::remove_image(&config.repository_name, language_id){
        return Json(RestResponse::err(e.to_string()))
    }
    Json(RestResponse::ok(()))
}


#[post("/run", data = "<code>")]
// #[post("/upload", data = "<zip>")]
pub fn run_code(config: &State<Config>, code: Form<Code>) -> Json<RestResponse<RunResult>> {
    let result = runner::run_code(&code, &config);
    if result.is_err() { 
        return Json(RestResponse::err(result.err().unwrap().to_string()))
    }
    return Json(RestResponse::ok(result.ok().unwrap()))
}