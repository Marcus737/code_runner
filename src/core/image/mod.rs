use std::collections::HashMap;
use std::path::{PathBuf};
use std::process::Command;
use log::{info};

use anyhow::{bail, Context, Result};
use crate::core::config::Config;
use crate::common::util::{foreach_file, run_cmd_get_result};
use crate::common::util::thread_util::ThreadPool;

#[derive(Debug)]
pub struct Image{
    pub _image_id:String,
    pub _language_id:String
}


pub fn get_created_image(prefix: &str) -> Result<HashMap<String, Image>>{
    let mut map:HashMap<String, Image> = HashMap::new();

    let mut command = Command::new("docker");
    command.arg("image")
        .arg("ls")
        .arg("--no-trunc")
        .arg(format!("--filter=reference={}", prefix))
        .arg("--format={{.Tag}} {{.ID}}");
    // info!(format!("{:?}", &command));

    run_cmd_get_result(&mut command)?
        .split("\n").for_each(
        |line|{
            if line.is_empty() {
                return ;
            }
            let line = &line[0..line.len() - 1];
            // info!(format!("line:{}", line));
            let spilt:Vec<&str> =  line.split(" ").collect();
            let language_id = spilt[0];
            let image_id = String::from(spilt[1])[7..].to_string();
            map.insert(String::from(language_id), Image{
                _image_id: image_id,
                _language_id: String::from(language_id),
            });
            info!("找到已创建镜像{:?}", map.get(language_id));
        }
    );

    Ok(map)
}

pub fn build_images(config:&Config) -> Result<HashMap<String, Image>> {

    // info!(format!("config:{:?}", &config));

    let mut map: HashMap<String, Image>;

    if config.use_created_image {
        map = get_created_image(&config.repository_name)?;
    }else {
        map = HashMap::new()
    }

    //遍历目录下的所有dockerfile
    let all_dockerfile =  get_all_dockerfile(&config.dockerfile_dir)?;

    let thread_pool = ThreadPool::new(3);
    let mut waiter = Vec::new();
    struct Entry{
        key: String,
        val: Image
    }

    for file in all_dockerfile {
        let repository_name = String::from(&config.repository_name);
        info!("{} {}", config.use_created_image, map.contains_key(&file.language_id));
        if config.use_created_image && map.contains_key(&file.language_id) {
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
            
            let output = run_cmd_get_result(&mut command)?;
            
            let image_id = String::from(&output[7..]).trim().to_string();
            
            info!("构建成功，镜像id为：{}", image_id);
            
            Ok(Box::new(
                Entry {
                    key: String::from(&language_id),
                    val: Image {
                        _image_id: image_id,
                        _language_id: String::from(&language_id),
                    }
                }
            ))
        });

        waiter.push(result);
    }
    
    for result in waiter {
        let entry_result = result?.recv()??.downcast::<Entry>();
        match entry_result {
            Ok(entry) => {
                map.insert(entry.key, entry.val);
            }
            Err(_err) => {
               bail!("cannot downcast entry_result");
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
fn get_all_dockerfile(dir: &str) -> Result<Vec<Dockerfile>>{
    let mut files:Vec<Dockerfile> = Vec::new();
    
    foreach_file(PathBuf::from(dir), 
                 &mut |dir_entry| {
        // println!("dir_entry:{:?}", dir_entry);
        if dir_entry.file_name().eq("Dockerfile") {
            let path_buf = dir_entry.path();
            
            let parent = path_buf.parent()
                .with_context(|| {"cannot not find parent dir".to_string()})?;

            let language_id = parent.file_name()
                .with_context(||{ "cannot get parent file name".to_string()})?
                .to_str()
                .with_context(||{ "osstr cannot to str".to_string()})?;

            let pwd = parent.to_str()
                .with_context(||{"cannot to string parent path".to_string()})?;

            info!("read a docker file:{:?}", pwd);
            files.push(Dockerfile{
                language_id: String::from(language_id),
                pwd: String::from(pwd),
            });
            
        }
                     
        Ok(())
                     
    })?;
    
    Ok(files)
}

#[test]
fn test_get_all_dockerfile(){
    let result = get_all_dockerfile("/home/panzi/rust_projects/code_runner/dockerfiles");
    println!("{:?}", result);
}