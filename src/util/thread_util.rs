use std::any::Any;
use std::fmt::Debug;
use std::sync::{Arc, LockResult, Mutex};
use std::sync::mpsc::{channel, Receiver, RecvError, Sender, SendError};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use log::{error, info};
use crate::util::thread_util::TaskType::EXIT;

type SendTrait = Result<Box<dyn Any + Send + 'static>,String>;

enum TaskType{
    NORMAL,
    EXIT
}


struct Task
{
    closure:  Option<Box<dyn FnOnce() -> SendTrait + Send + 'static>>,
    task_type: TaskType,
    result_sender: Option<Sender<SendTrait>>
}

pub struct ThreadPool {
    core_size: u32,
    workers: Vec<Worker>,
    sender: Sender<Task>,

}

struct Worker{
    id: usize,
    job: JoinHandle<()>
}

impl Worker{
    fn new(id: usize, recv: Arc<Mutex<Receiver<Task>>>) -> Worker {
        // println!("cur id : {}", id);
        let thread = thread::spawn( move || {
            loop {
                let recv = match recv.lock() {
                    Ok(r) => { r }
                    Err(err) => {
                        error!("{}", err.to_string());
                        break
                    }
                };
                
                let task =  match recv.recv() {
                    Ok(r) => {r}
                    Err(err) => {
                        error!("{}", err.to_string());
                        break
                    }
                };
                
                match task.task_type {
                    TaskType::NORMAL => {
                        println!("Worker {id} got a job; executing.");
                        info!("Worker {id} got a job; executing.");
                        let fun = task.closure;
                        
                        let fun = match fun {
                            None => {
                                continue;
                            }
                            Some(f) => {f}
                        };
                        
                        let result = fun();
                        match task.result_sender {
                            None => {}
                            Some(sender) => {
                                match sender.send(result) {
                                    Ok(_) => {}
                                    Err(error) => {
                                        error!("{}" ,error.to_string());
                                    }
                                };
                            }
                        }
                    }
                    EXIT => {
                        break;
                    }
                }
            }
            info!("Worker {id} is shutdown.");
            // println!("Worker {id} is shutdown.");
        });
        Worker{ id, job: thread }
    }
}



impl ThreadPool {
    
    pub fn execute<F>(&self, f: F) -> Result<Receiver<SendTrait>,String>
    where 
        F: FnOnce() -> SendTrait + Send + 'static 
    {
        let (sender, recv) = channel::<SendTrait>();
        match self.sender.send(Task {
            closure: Some(Box::new(f)),
            task_type: TaskType::NORMAL,
            result_sender: Some(sender),
        }) {
            Ok(_) => {}
            Err(err) => {
                return Err(err.to_string());
            }
        };
        Ok(recv)
    }
    
    
    pub fn new(core_size: u32) -> ThreadPool{
        let mut handles = Vec::with_capacity(core_size as usize);

        let (sender, recv) = channel::<Task>();

        let recv = Arc::new(Mutex::new(recv));

        for i in 0..core_size {
  
            handles.push(Worker::new(i as usize, recv.clone()))
        }

        ThreadPool{
            core_size,
            workers: handles,
            sender
        }
    }
    
    pub fn shutdown(&self){
        for _i in 0..self.core_size {
            self.sender.send(Task{
                closure: None,
                task_type: EXIT,
                result_sender: None,
            }).unwrap()
        }
    }
}

#[test]
fn test_thread_pool(){
    let pool = ThreadPool::new(3);
    let  mut v = Vec::new();
    for i in 1..50 {
        let receiver = pool.execute(move || {
            // sleep(Duration::from_millis(100 * i));
            Ok(Box::new(i.to_string()))
        });
        v.push(receiver);
    }
    for receiver in v {
        let r = receiver.unwrap().recv().unwrap().unwrap();
        let x = r.downcast_ref::<String>().unwrap();
        println!("{}", x);
    }

    pool.shutdown()
}
