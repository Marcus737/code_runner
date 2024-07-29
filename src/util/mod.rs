use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, SystemTime};

pub fn run_cmd_get_result(cmd: &mut Command) -> Result<String, String>{
    let res = cmd.output().unwrap();
    if res.status.success(){
        let output = String::from_utf8(res.stdout).unwrap().trim().to_string();
        Ok(output)
    }else {
        let output = String::from_utf8(res.stderr).unwrap();
        Err(output)
    }
}


pub struct CmdResult{
    pub spend_time:Duration, //seconds
    pub result: String,
    // pub avg_mem: u64, //kb
}

pub fn run_cmd_with_time_limit(cmd: &mut Command, keep_time: Duration) -> Result<CmdResult, String>{
    let mut child = cmd
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let millis_time = keep_time.as_millis();
    let mut front = SystemTime::now();
    let mut spend_time:u128 = 0;
    loop {
        if spend_time >= millis_time {
            child.kill().expect("kill self");
            return Err("exec command time out".parse().unwrap());
        }
        match child.try_wait() {
            Ok(Some(status) ) => {

                let res = child.wait_with_output().unwrap();
                return if status.success() {
                    let output = String::from_utf8(res.stdout).unwrap().trim().to_string();
                    Ok(CmdResult{
                        spend_time: Duration::from_millis(spend_time as u64),
                        result:output
                    })
                } else {
                    let output = String::from_utf8(res.stderr).unwrap();
                    Err(output)
                }
            }
            Err(err) => {
                return Err(err.to_string());
            }
            Ok(None)=>{
                //5ms
                sleep(Duration::new(0, 5000000));
                let now = SystemTime::now();
                let diff = now.duration_since(front).unwrap().as_millis();
                front = now;
                spend_time += diff;
            }
        }
    }
}

