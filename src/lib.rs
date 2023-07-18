use std::{thread, sync::{mpsc, Arc, Mutex}};
pub struct ThreadPool {
    // 使用 std::thread::spawn() 函数创建一个新线程
    // 通过 JoinHandle 对象的 join() 方法，可以阻塞当前线程，等待与其关联的线程执行完毕，并返回线程的执行结果
    // 使用 JoinHandle 对象的 is_completed() 方法可以检查与其关联的线程是否已经完成
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size:usize) -> ThreadPool{
        assert!(size>0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size{
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool{workers, sender:Some(sender)}
    }

    pub fn execute<F>(&self, f:F)
    where
        F:FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool{
    fn drop(&mut self){
        drop(self.sender.take());

        for worker in &mut self.workers{
            println!("shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker{
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

impl Worker {
    fn new(id:usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move|| loop{
            let message = receiver.lock()
                                        .unwrap()
                                        .recv();                                        
            match message {
                Ok(job) => {
                    println!("worker {id} got a job; executing.");
                    job();
                }
                Err(_) => {
                    println!("worker {id} disconnected; shutting down");
                    break;
                }
            }                        

        });
        Worker {id, thread:Some(thread)}
    }
    
}