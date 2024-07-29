pub mod java_tape;

mod common_record;

use std::any::Any;
use std::collections::HashMap;
use std::slice::Iter;
use tklog::info;
use crate::Code;
use crate::tape::common_record::CommonRecord;

pub struct ContextMap{
    context: HashMap<String, Box<dyn Any>>
}
impl ContextMap{
    pub fn set_val(&mut self, key: &str, val: Box<dyn Any>){
        self.context.insert(String::from(key), val);
    }

    pub fn get_val<T: Any>(&mut self, key: &str) -> Option<&mut T> {
        let option: &mut Box<dyn Any> = self.context.get_mut(key).unwrap();
        option.downcast_mut::<T>()
    }

    pub fn remove(&mut self, key: &str){
        self.context.remove(key);
    }

    fn new() -> ContextMap {
        ContextMap{
            context: HashMap::new()
        }
    }
}

pub trait Tape{
    fn add(&mut self, record: Box<Record>);
    fn get_iter(&self) -> Box<Iter<Box<Record>>>;
}

pub type Record = dyn Fn(&mut ContextMap) -> Result<(), String>;

pub struct TapePlayer {
    tape: Box<dyn Tape>,
    context_map: ContextMap
}

pub struct PlayResult{
    code: Code,
    status: String,
    output: String,

}

impl TapePlayer {
    pub fn new(tape: Box<dyn Tape>) -> TapePlayer {
        TapePlayer {
            tape,
            context_map: ContextMap::new()
        }
    }

    pub fn play(&mut self, code: Code){
        let mut step = 0;
        CommonRecord::init(code)(&mut self.context_map).expect("初始化容器失败");
        for record in self.tape.get_iter() {
            step += 1;
            info!(format!("执行第{}步", &step));
            // let record = record.deref();
            match record(&mut self.context_map) {
                Ok(_) => {
                    info!(format!("第{}步执行成功", &step));
                }
                Err(err) => {
                    println!("出错：{}", err);
                    break;
                }
            }
        }
        CommonRecord::kill()(&mut self.context_map).expect("关闭容器失败");
    }
}