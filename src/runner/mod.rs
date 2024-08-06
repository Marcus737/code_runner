use std::collections::HashMap;
use std::process::Command;
use log::{ error, info};
use uuid::Uuid;
use crate::Code;
use crate::config::Config;
use crate::util::{build_container_exec_command, run_cmd_get_result};

#[derive(Debug)]
pub struct RunResult{
    pub status: String,
    pub use_time: String,
    pub memory: String,
    pub output: String
}

///运行代码
pub fn run_code(code: &Code, config: &Config) -> Result<RunResult, String> {
    //创建容器并得到容器id
    let container_id = match run_container(&code.image_name, &config.container_prefix){
        Ok(container_id) => {
            container_id
        }
        Err(err) => {
            return Err(err);
        }
    };

    // //切换到指定目录
    // info!("change dir to {}", config.container_workdir);
    // let mut command = build_container_exec_command(
    //     &container_id,
    //    vec!["cd", &config.container_workdir]);
    //
    // match run_cmd_get_result(&mut  command) {
    //     Ok(_) => {}
    //     Err(err) => {
    //         return Err(err);
    //     }
    // }

    info!("run_container成功，容器id：{}", &container_id);
    //写入代码到容器里
    info!("写入代码到容器里");
    match copy_str_to_container(&container_id, &config.src_code_filename, &code.code_string) {
        Ok(_) => {}
        Err(err) => {
            let _ = kill_container(&container_id);
            return Err(err);
        }
    }

    //写入输入到容器里
    info!("写入输入到容器里");
    match copy_str_to_container(&container_id, &config.input_filename, &code.input) {
        Ok(_) => {}
        Err(err) => {
            let _ = kill_container(&container_id);
            return Err(err);
        }
    }

    //写入执行脚本到容器里
    info!("写入执行脚本到容器里");
    let script_name = format!("{}.sh", &code.language_id);
    let script_path = format!("{}{}/{}", &config.dockerfile_dir, &code.language_id, &script_name);
    match copy_file_to_container(&container_id, &script_name, &script_path) {
        Ok(_) => {}
        Err(err) => {
            let _ = kill_container(&container_id);
            return Err(err)
        }
    }
    //为脚本添加执行权限
    info!("为脚本添加执行权限");
    match add_executable_for_file(&container_id, &script_name) {
        Ok(_) => {
            // debug!("{}", out);
        }
        Err(err) => {
            let _ = kill_container(&container_id);
            return Err(err);
        }
    }

    info!("开始执行");
    //执行脚本,得到运行结果
    let result = run_script(&container_id, &script_name, &config.run_result_filename, &code.time_limit, &code.memory_limit);

    let _ = kill_container(&container_id);
    result
}


///关闭容器
fn kill_container(container_id: &str) -> Result<String, String> {
    // Ok("".to_string())
    let mut command = Command::new("docker");
    command.arg("rm")
        .arg("-f")
        .arg(container_id);
    
    match run_cmd_get_result(&mut command) {
        Ok(_output) => {
            info!("kill succeed");
            Ok("killed".parse().unwrap())
        }
        Err(err_output) => {
            Err(err_output)
        }
    }
}

///为文件添加可执行性
fn add_executable_for_file(container_id: &str, script_name: &str) -> Result<String, String> {
    let mut command= build_container_exec_command(
        container_id,
        vec![&format!("chmod +x {}", script_name)]);
    run_cmd_get_result(&mut command)
}


///运行容器脚本
fn run_script(container_id: &str, script_name: &str, result_filename: &str, time_limit: &u64, memory_limit: &u64) -> Result<RunResult, String> {
    let mut command= build_container_exec_command(
        container_id,
        vec![&format!("./{} {} {}", script_name, time_limit, memory_limit)]);
    let output = match run_cmd_get_result(&mut command) {
        Ok(output) => {
            output
        }
        Err(err) => {
            error!("{}", err);
            return Err(err)
        }
    };
    //从容器读取读取文件内容
    return match read_container_file(container_id, result_filename) {
        Ok(res_file) => {
            let map: HashMap<String, String> = parse_res_file(res_file);
            let status = match map.get("status") {
                None => {
                    return Err("status不存在".to_string())
                }
                Some(status) => {
                    status
                }
            };
            if status.eq("OK") {
                let use_time = map.get("wall_time").unwrap();
                let max_mem = map.get("memory").unwrap();
                Ok(RunResult {
                    status: String::from(status),
                    use_time: format!("{}ms", use_time.to_string()),
                    memory: format!("{}KB", max_mem.to_string()),
                    output,
                })
            } else {
               Ok(
                   RunResult {
                       status: String::from(status),
                       use_time: "".to_string(),
                       memory: "".to_string(),
                       output,
                   }
               )
            }
        }
        Err(err) => {
            Err(err)
        }
    }
}

///解析结果文件
fn parse_res_file(content: String) -> HashMap<String, String> {
    // info!("content:{}", content);
    let mut map = HashMap::new();
    content.split("\n")
        .for_each(|line|{
            let line = line.trim();
            if line.is_empty() { return ; }
            let collect:Vec<&str> = line.split("=").collect();
            if collect.len() != 2 { return ; }
            map.insert(String::from(collect[0]), String::from(collect[1]));
        });
    map
}


///读取容器文件
fn read_container_file(container_id: &str, filename: &str) -> Result<String, String>{
    let mut command = build_container_exec_command(
        container_id,
        vec![&format!("cat {}", filename)]
    );
    run_cmd_get_result(&mut command)
}


///运行容器
fn run_container(image_id: &str, code_prefix: &str) -> Result<String, String> {
    let container_name = format!("{}{}", code_prefix, Uuid::new_v4());
    //从docker中创建镜像的容器,并把容器id放进context_map
    info!("container name:{}", &container_name.as_str());
    let mut command = Command::new("docker");
    command.arg("container")
        .arg("run")
        .arg("--rm")
        .arg("--detach")
        .arg("--interactive")
        .arg(&format!("--name={}", container_name))
        .arg(image_id);

    run_cmd_get_result(&mut command)
}

///将字符串复制到容器
fn copy_str_to_container(container_id: &str, filename: &str, content: &str) -> Result<String, String> {
    let mut command = build_container_exec_command(
        container_id,
       vec![&format!("echo '{}' > {}", content, filename)]
    );
    run_cmd_get_result(&mut command)
}


///
/// 将文件复制的到容器
fn copy_file_to_container(container_id: &str, filename: &str, file_path: &str) -> Result<String, String> {
    let mut command = Command::new("docker");
    command.arg("cp")
        .arg(file_path)
        .arg(format!("{}:{}", container_id, filename));
    run_cmd_get_result(&mut command)
}