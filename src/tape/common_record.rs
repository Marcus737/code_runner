use std::process::Command;
use tklog::info;
use uuid::Uuid;
use crate::Code;
use crate::tape::Record;
use crate::util::run_cmd_get_result;

pub struct CommonRecord;
impl CommonRecord{

    pub fn init(code: Code) -> Box<Record> {
        info!("init");
        Box::new(move |con| {
            let container_name = format!("code_runner_{}", Uuid::new_v4());

            //从docker中创建镜像的容器,并把容器id放进context_map
            info!(&code.image_id);
            let mut command = Command::new("docker");
            command.arg("container")
                .arg("run")
                .arg("--rm")
                .arg("--detach")
                .arg("--interactive")
                .arg(&format!("--name={}", container_name))
                .arg(&code.image_id);

            con.set_val("code", Box::new(code.clone()));

            match run_cmd_get_result(&mut command) {
                Ok(output) => {
                    let container_id = output;
                    info!(format!("container_id:{}", container_id));
                    con.set_val("container_id", Box::new(container_id));
                    Ok(())
                }
                Err(err_output) => {
                    Err(err_output)
                }
            }

            //写入源文件


        })
    }

    pub fn kill() -> Box<Record> {
        info!("kill");
        Box::new(move |con| {
            let container_id = con.get_val::<String>("container_id").unwrap().clone();
            let mut command = Command::new("docker");
            command.arg("rm")
            .arg("-f")
            .arg(&container_id);

            match run_cmd_get_result(&mut command) {
                Ok(_output) => {
                    info!("kill succeed");
                    Ok(())
                }
                Err(err_output) => {
                    Err(err_output)
                }
            }
        })
    }
}