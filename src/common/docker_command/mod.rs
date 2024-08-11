use std::process::Command;
use uuid::Uuid;
use crate::common::util;
use anyhow::Result;

pub fn build_container_exec_command(container_id: &str, args: Vec<&str>) -> Command{
    let mut command = Command::new("docker");
    command.arg("container")
        .arg("exec")
        .arg(container_id)
        .arg("sh")
        .arg("-c")
        .args(args);
    command
}

pub fn read_container_file(container_id: &str, filename: &str) -> Result<String>{
    let mut command = build_container_exec_command(
        container_id,
        vec![&format!("cat {}", filename)]
    );
    util::run_cmd_get_result(&mut command)
}

pub fn kill_container(container_id: &str) -> Result<String> {
    // Ok("".to_string())
    let mut command = Command::new("docker");
    command.arg("rm")
        .arg("-f")
        .arg(container_id);

    util::run_cmd_get_result(&mut command)
}

///为文件添加可执行性
pub(crate) fn add_executable_for_file(container_id: &str, script_name: &str) -> Result<String> {
    let mut command= build_container_exec_command(
        container_id,
        vec![&format!("chmod +x {}", script_name)]);
    util::run_cmd_get_result(&mut command)
}

///运行容器
pub fn run_container(image_id: &str, code_prefix: &str) -> Result<String> {
    let container_name = format!("{}{}", code_prefix, Uuid::new_v4());
    //从docker中创建镜像的容器,并把容器id放进context_map
    // info!("container name:{}", &container_name.as_str());
    let mut command = Command::new("docker");
    command.arg("container")
        .arg("run")
        .arg("--rm")
        .arg("--detach")
        .arg("--interactive")
        .arg(&format!("--name={}", container_name))
        .arg(image_id);

    util::run_cmd_get_result(&mut command)
}

///将字符串复制到容器
pub fn copy_str_to_container(container_id: &str, filename: &str, content: &str) -> Result<String> {
    let mut command = build_container_exec_command(
        container_id,
        vec![&format!("echo '{}' > {}", content, filename)]
    );
    util::run_cmd_get_result(&mut command)
}


///
/// 将文件复制的到容器
pub fn copy_file_to_container(container_id: &str, filename: &str, file_path: &str) -> Result<String> {
    let mut command = Command::new("docker");
    command.arg("cp")
        .arg(file_path)
        .arg(format!("{}:{}", container_id, filename));
    util::run_cmd_get_result(&mut command)
}

#[test]
pub fn test_build_container_exec_command(){
    let command = build_container_exec_command("123", vec!["ls", "-l"]);
    println!("{:?}", &command);
    assert!(command.get_program().eq("docker"))
}

pub fn build_image(pwd: &str,repository_name: &str, language_id: &str) -> Result<String>{
    let mut command = Command::new("docker");
    command
        .current_dir(pwd)
        .arg("image")
        .arg("build")
        .arg("--quiet")
        .arg(format!("--tag={}:{}", repository_name, language_id))
        .arg(".");

    let output = util::run_cmd_get_result(&mut command)?;

    let image_id = String::from(&output[7..]).trim().to_string();

    info!("构建成功，镜像id为：{}", image_id);
    
    Ok(image_id)
}

pub fn remove_image(repository_name: &str, language_id: &str) ->  Result<()>{
    let mut command = Command::new("docker");
    command
        .arg("image")
        .arg("rm")
        // .arg("-f")
        .arg(format!("{}:{}", repository_name, language_id));

    debug!("{:?}", &command);

    util::run_cmd_get_result(&mut command)?;
    
    Ok(())
}