use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use log::{error, info};
use crate::config::Config;
use crate::util::{foreach_file, run_cmd_get_result};
use crate::util::thread_util::ThreadPool;

#[derive(Debug)]
pub struct Image{
    pub image_id:String,
    pub language_id:String
}

pub fn get_created_image(prefix: &str) -> Result<HashMap<String, Image>, String>{
    let mut map:HashMap<String, Image> = HashMap::new();

    let mut command = Command::new("docker");
    command.arg("image")
        .arg("ls")
        .arg("--no-trunc")
        .arg(format!("--filter=reference={}", prefix))
        .arg("--format='{{.Tag}} {{.ID}}'");
    // info!(format!("{:?}", &command));

    match run_cmd_get_result(&mut command){
        Ok(output) => {
            // info!(format!("输出：{}", &output));
            output.split("\n").for_each(
                |line|{
                    if line.is_empty() {
                        return ;
                    }
                    let line = &line[1..line.len() - 1];
                    // info!(format!("line:{}", line));
                    let spilt:Vec<&str> =  line.split(" ").collect();
                    let language_id = spilt[0];
                    let image_id = String::from(spilt[1])[7..].to_string();
                    map.insert(String::from(language_id), Image{
                        image_id,
                        language_id: String::from(language_id),
                    });
                    info!("找到已创建镜像{:?}", map.get(language_id));
                }
            )
        }
        Err(err) => {
            error!("{}" ,err);
            return Err(err);
        }
    }
    Ok(map)
}

pub fn build_images(config:&Config) -> Result<HashMap<String, Image>,String> {

    // info!(format!("config:{:?}", &config));

    let mut map: HashMap<String, Image>;

    if config.use_created_image {
        map = match get_created_image(&config.repository_name) {
            Ok(m) => {
                m
            }
            Err(err) => {
                return Err(err);
            }
        }
    }else {
        map = HashMap::new()
    }

    //遍历目录下的所有dockerfile
    let all_dockerfile = match get_all_dockerfile(&config.dockerfile_dir) {
        Ok(f) => {f}
        Err(err) => {
            return Err(err);
        }
    };

    let thread_pool = ThreadPool::new(3);
    let mut waiter = Vec::new();
    struct Entry{
        key: String,
        val: Image
    }

    for file in all_dockerfile {
        let repository_name = String::from(&config.repository_name);
        let exist =  map.contains_key( &file.language_id);
        if config.use_created_image && exist {
            info!("已存在镜像{}", &file.language_id);
            continue;
        }
        info!("begin build {}", file.pwd);

        let result = thread_pool.execute( move || {
            info!("开始构建镜像{:?}", &file);
            let language_id = file.language_id;
            let mut command = Command::new("docker");
            command
                .current_dir(file.pwd)
                .arg("image")
                .arg("build")
                .arg("--quiet")
                .arg(format!("--tag={}:{}", repository_name, &language_id))
                .arg(".");

            return match run_cmd_get_result(&mut command) {
                Ok(output) => {
                    let image_id = String::from(&output[7..]).trim().to_string();
                    info!("构建成功，镜像id为：{}", image_id);
                    let image = Image {
                        image_id,
                        language_id: String::from(&language_id),
                    };
                    Ok(Box::new(
                        Entry {
                            key: String::from(&language_id),
                            val: image
                        }
                    ))
                    // map.insert(String::from(&language_id), image);
                }
                Err(err) => {
                    error!("{}" ,err);
                    Err(format!("构建镜像失败：{}", err))
                }
            }
        });

        waiter.push(result);
    }
    for result in waiter {
        let recv =  match result {
            Ok(s) => {s}
            Err(err) => {
                return Err(err);
            }
        };
        match recv.recv() {
            Ok(r) => {
                match r {
                    Ok(b) => {
                        match b.downcast::<Entry>() {
                            Ok(e) => {
                                info!("build finish:{}", &e.key);
                                map.insert(e.key, e.val);
                            }
                            Err(_err) => {
                                return Err("cast fail".to_string());
                            }
                        }
                    }
                    Err(err) => {
                        return Err(err.to_string())
                    }
                }
            }
            Err(err) => {
                return Err(err.to_string());
            }
        }
    }

    thread_pool.shutdown();

    Ok(map)
}


#[derive(Debug)]
struct Dockerfile{
    language_id: String,
    pwd: String
}

///
/// 遍历dockerfile目录
/// 返回dockerfile列表
fn get_all_dockerfile(dir: &str) -> Result<Vec<Dockerfile>, String>{
    let mut files:Vec<Dockerfile> = Vec::new();
    match foreach_file(PathBuf::from(dir), &mut files,|ff, dir_entry| {
        // println!("dir_entry:{:?}", dir_entry);
        if dir_entry.file_name().eq("Dockerfile") {
            let pwd = dir_entry.path();
            match pwd.parent() {
                None => {
                    return Err("cannot not find parent dir".to_string());
                }
                Some(parent) => {
                    let language_id = match parent.file_name() {
                        None => {
                            return Err("cannot get dir name".to_string());
                        }
                        Some(dir_name) => {
                            String::from(match dir_name.to_str() {
                                None => {
                                    return Err("osstr cannot to str".to_string());
                                }
                                Some(s) => {
                                    String::from(s)
                                }
                            })
                        }
                    };
                    let pwd = match parent.to_str() {
                        None => {
                            return Err("cannot tostring parent".to_string());
                        }
                        Some(p) => {
                            String::from(p)
                        }
                    };
                    info!("read a docker file:{:?}", pwd);
                    ff.push(Dockerfile{
                        language_id,
                        pwd
                    });

                }
            }
        }
        Ok(())
    }) {
        Ok(_) => {}
        Err(err) => {
            return Err(err);
        }
    }
    Ok(files)
}

#[test]
fn test_get_all_dockerfile(){
    let result = get_all_dockerfile("/home/panzi/rust_projects/code_runner/dockerfiles");
    println!("{:?}", result);
}