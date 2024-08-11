pub mod thread_util;
pub mod zip_util;

use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::process::Command;
use anyhow::{bail, Result};

pub fn run_cmd_get_result(cmd: &mut Command) -> Result<String>{
    let res = cmd.output()?;
    if res.status.success(){
        let output = String::from_utf8(res.stdout)?.trim().to_string();
        Ok(output)
    }else {
        let output = String::from_utf8(res.stderr)?;
        // Err(output)
        bail!(output)
    }
}


///
/// 递归遍历dir_path下所有文件
pub fn foreach_file<T>(dir_path: PathBuf, consumer: &mut T) -> Result<()>
    where T: FnMut(DirEntry) -> Result<()>
{
    let read_dir = fs::read_dir(dir_path)?;
    for file in read_dir {
        let dir_entry = file?;
        // println!("{:?}", dir_entry);
        let metadata = dir_entry.metadata()?;
        if metadata.is_dir() {
            foreach_file(dir_entry.path(), consumer)?
        }else {
            consumer(dir_entry)?
        }
    }
    Ok(())
}

#[test]
fn test_foreach_file(){
    let s  = "/home/panzi/rust_projects/code_runner/dockerfiles";
    let result = foreach_file(PathBuf::from(s), &mut |dir_entry|{
        println!("{:?}", dir_entry);
        Ok(())
    });
    println!("{:?}", result);
}


