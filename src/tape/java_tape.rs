use std::process::Command;
use std::slice::Iter;
use std::time::Duration;
use tklog::info;
use crate::Code;
use crate::util::{run_cmd_get_result, run_cmd_with_time_limit};
use super::*;

pub struct JavaTape {
    records: Vec<Box<Record>>
}

impl JavaTape{
    pub fn new() -> JavaTape{
        let mut vec:Vec<Box<Record>> = Vec::new();
        vec.push(write_code());
        vec.push(compile_main());
        vec.push(run_main());
        JavaTape{
            records:vec
        }
    }
}

impl Tape for JavaTape{
    fn add(&mut self, record: Box<Record>) {
        self.records.push(record);
    }

    fn get_iter(&self) -> Box<Iter<Box<Record>>> {
        Box::from(self.records.iter())
    }
}

 fn write_code() -> Box<Record> {
     Box::new(
         |con|{
             info!("write_code");
             let res = con.get_val::<String>("container_id").unwrap().clone();
             let code = con.get_val::<Code>("code").unwrap();

             let mut command = Command::new("docker");
             command.arg("container")
                 .arg("exec")
                 .arg(res)
                 .arg("sh")
                 .arg("-c")
                 .arg(format!("echo '{}' > Main.java", &code.code_string))
                 .output()
                 .unwrap();

             match run_cmd_get_result(&mut command)
             {
                 Ok(_) => {
                     info!("write_code succeed");
                     Ok(())
                 }
                 Err(err_output) => {
                     return Err(err_output);
                 }
             }
         }
     )
 }

fn compile_main() -> Box<Record> {
    Box::new(|con|{
        info!("compile_main");
        let res = con.get_val::<String>("container_id").unwrap();
        let mut command = Command::new("docker");

        command
            .arg("container")
            .arg("exec")
            .arg(res)
            .arg("sh")
            .arg("-c")
            .arg("javac -encoding utf-8 Main.java");

        match run_cmd_get_result(&mut command) {
            Ok(output) => {
                //javac编译出错却返回成功？
                if output.len() > 0 {
                    return Err(output);
                }
                info!("javac compile succeed");
                Ok(())
            }
            Err(err) => {
                return Err(err);
            }
        }
    })
}

fn run_main() -> Box<Record> {
    Box::new(|con|{
        info!("run_main");
        let res = con.get_val::<String>("container_id").unwrap().clone();
        let code = con.get_val::<Code>("code").unwrap();
        let mut command =  Command::new("docker");
        command
            .arg("container")
            .arg("exec")
            .arg(res)
            .arg("sh")
            .arg("-c")
            .arg(format!("echo '{}' | java -Xmx{}m Main", code.input, code.memory_limit));

        let duration = Duration::from_millis(code.time_limit);
        match run_cmd_with_time_limit(&mut command, duration) {
            Ok(res) => {
                info!("java run succeed");
                info!(format!("使用时间：{:?} 程序输出：{}", res.spend_time, res.result));
                Ok(())
            }
            Err(err) => {
                return Err(err);
            }
        }
    })
}