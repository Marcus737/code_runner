use std::collections::HashMap;
use log::{ info};
use anyhow::{Context, Result};
use crate::core::config::Config;
use crate::common::{Code, docker_command, RunResult};
use crate::common::util;



///运行代码
pub fn run_code(code: &Code, config: &Config) -> Result<RunResult> {
    //创建容器并得到容器id
    let image_name = format!("{}:{}", config.repository_name, code.language_id);
    let container_id =  docker_command::run_container(&image_name, &config.container_prefix)?;
    let result = run_code_inner(&container_id, code, config);
    docker_command::kill_container(&container_id)?;
    result
}


fn run_code_inner(container_id: &str, code: &Code, config: &Config) -> Result<RunResult> {
    
    info!("run_container成功，容器id：{}", &container_id);
    //写入代码到容器里
    info!("写入代码到容器里");
    docker_command::copy_str_to_container(&container_id, &config.src_code_filename, &code.code_string)?;

    //写入输入到容器里
    info!("写入输入到容器里");
    docker_command::copy_str_to_container(&container_id, &config.input_filename, &code.input)?;

    //写入执行脚本到容器里
    info!("写入执行脚本到容器里");
    let script_name = format!("{}.sh", &code.language_id);
    let script_path = format!("{}{}/{}", &config.dockerfile_dir, &code.language_id, &script_name);
    docker_command::copy_file_to_container(&container_id, &script_name, &script_path)?;

    //为脚本添加执行权限
    info!("为脚本添加执行权限");
    docker_command::add_executable_for_file(&container_id, &script_name)?;

    info!("开始执行");
    //执行脚本,得到运行结果
    let run_result = run_script(&container_id, &script_name, &config.run_result_filename, &code.time_limit, &code.memory_limit)?;
    Ok(run_result)
    
}




///运行容器脚本
fn run_script(container_id: &str, script_name: &str, result_filename: &str, time_limit: &u64, memory_limit: &u64) -> Result<RunResult> {
    let mut command= docker_command:: build_container_exec_command(
        container_id,
        vec![&format!("./{} {} {}", script_name, time_limit, memory_limit)]);
    
    let output = util::run_cmd_get_result(&mut command)?;
    //从容器读取读取文件内容
    let res_file = docker_command::read_container_file(container_id, result_filename)?;
    
    let map: HashMap<String, String> = parse_res_file(res_file);

    let status =  map.get("status")
        .with_context(|| {"status不存在".to_string()})?;
    
    if status.eq("OK") {

        let use_time = map.get("wall_time")
            .with_context(|| {"wall_time不存在".to_string()})?;

        let max_mem = map.get("memory")
            .with_context(|| {"memory不存在".to_string()})?;

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






