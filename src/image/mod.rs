use std::collections::HashMap;
use std::fs;
use std::process::Command;
use tklog::{error, info};
use crate::config::Config;
use crate::util::run_cmd_get_result;

#[derive(Debug)]
pub struct Image{
    pub image_id:String,
    pub language_id:String
}

pub fn get_created_image() -> HashMap<String, Image>{
    let mut map:HashMap<String, Image> = HashMap::new();

    let mut command = Command::new("docker");
    command.arg("image")
        .arg("ls")
        .arg("--no-trunc")
        .arg(format!("--filter=reference={}", "code_runner"))
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
                    info!(format!("找到已创建镜像{:?}", map.get(language_id)));
                }
            )
        }
        Err(err) => {
            error!(err);
            panic!()
        }
    }
    map
}

pub fn build_images(config:Config) -> HashMap<String, Image> {

    // info!(format!("config:{:?}", &config));

    let mut map:HashMap<String, Image>;

    if config.use_created_image {
        map = get_created_image();
    }else {
        map = HashMap::new();
    }

    //遍历目录下的所有dockerfile
    let all_dockerfile = get_all_dockerfile(&config.dockerfile_dir);

    for file in all_dockerfile {
        let language_id = file.language_id;
        if config.use_created_image && map.contains_key(&language_id) {
            info!(format!("已存在镜像{}", &language_id));
            continue;
        }

        info!(format!("开始构建镜像{}", &language_id));

        let mut command = Command::new("docker");
        command
            .current_dir(file.pwd)
            .arg("image")
            .arg("build")
            .arg("--quiet")
            .arg(format!("--tag=code_runner:{}", &language_id))
            .arg(format!("--file={}", &language_id))
            .arg(".")
            .output()
            .expect("执行构建镜像失败");

        match run_cmd_get_result(&mut command) {
            Ok(output) => {
                let image_id = String::from(&output[7..]).trim().to_string();
                info!(format!("构建成功，镜像id为：{}", image_id));
                let image = Image {
                    image_id,
                    language_id: String::from(&language_id),
                };
                map.insert(String::from(&language_id), image);
            }
            Err(err) => {
                error!(err);
                panic!("获取镜像id失败");
            }
        }
    }
    map
}


struct Dockerfile{
    language_id: String,
    pwd: String
}

fn get_all_dockerfile(dir: &str) -> Vec<Dockerfile>{
    let mut files:Vec<Dockerfile> = Vec::new();
    let read_dir = fs::read_dir(dir).unwrap();
    for file in read_dir {
        let filename = String::from(file.unwrap().file_name().to_str().unwrap());
        files.push(
            Dockerfile{
                language_id: String::from(filename),
                pwd: String::from(dir),
            }
        );
    }
    files
}