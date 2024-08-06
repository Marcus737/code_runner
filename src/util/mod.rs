pub mod thread_util;

use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::process::Command;

pub fn run_cmd_get_result(cmd: &mut Command) -> Result<String, String>{
    let res = match cmd.output() {
        Ok(r) => {r}
        Err(err) => {
            return Err(err.to_string());
        }
    };
    if res.status.success(){
        let output = String::from_utf8(res.stdout).unwrap().trim().to_string();
        Ok(output)
    }else {
        let output = String::from_utf8(res.stderr).unwrap();
        Err(output)
    }
}


///
/// 递归遍历dir_path下所有文件
pub fn foreach_file<T>(dir_path: PathBuf, arg: &mut T, consumer: fn(&mut T, DirEntry)->Result<(), String>) -> Result<(), String>{

    match fs::read_dir(dir_path) {
        Ok(read_dir) => {
            for file in read_dir {
                match file {
                    Ok(dir_entry) => {
                        // println!("{:?}", dir_entry);
                        match dir_entry.metadata() {
                            Ok(metadata) => {
                                if metadata.is_dir() {
                                    match foreach_file(dir_entry.path(), arg, consumer) {
                                        Ok(_) => {}
                                        Err(err) => {
                                            return Err(err);
                                        }
                                    }
                                }else {
                                    match consumer(arg, dir_entry) {
                                        Ok(_) => {}
                                        Err(err) => {
                                            return Err(err);
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                return Err(err.to_string());
                            }
                        }
                    }
                    Err(err) => {
                        return Err(err.to_string());
                    }
                }
            }
            Ok(())
        }
        Err(err) => {
            return Err(err.to_string());
        }
    }
}

#[test]
fn test_foreach_file(){
    let mut s  = "/home/panzi/rust_projects/code_runner/dockerfiles";
    let result = foreach_file(PathBuf::from(s), &mut s ,|_str,dir_entry| {
        println!("{:?}", dir_entry);
        Ok(())
    });
    println!("{:?}", result);
}

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

#[test]
fn test_build_container_exec_command(){
    let command = build_container_exec_command("123", vec!["ls", "-l"]);
    println!("{:?}", &command);
    assert!(command.get_program().eq("docker"))
}

