use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

/// ThreadPool 实现的本质实现还是消费者-生成者
/// 只不过在Rust中线程之间交流的方式倾向于消息传递而不是共享内存（即消息队列那一套）
/// ThreadPool 中的线程workers作为 消费者 不断监听并消费 消息
/// ThreadPool 的函数execute作为 生产者 不断生成 消息
/// 在此ThreadPool的消息为 要执行的闭包
/// 
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,  // 为了能够从ThreadPool的可变引用中取出sender的所有权需要将sender实现为Option
}

/** Job 定义为 要执行的闭包 的类型
 * || {
        handle_connection(stream);
    }
 * 我们要执行的闭包如上， 没有参数，没有返回值，会消耗掉外部变量stream
 * 所以需要使用FnOnce trait
 * 'static 表示其不依赖外部非全局借用
 * Send表示其可安全地在多线程下传输到另一个多线程
 * */
type Job = Box<dyn FnOnce() + 'static + Send>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        // Vec::with_capacity 在知道固定大小时使用，速度比Vec::new更块
        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();

        // rust的消息传递实现为多生成者，单消费者；为了多个线程能够使用receiver
        // 需要使用到Arc引用receiver, 同时receiver作为共享的资源，需要锁 Mutex
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender: Some(sender) }
    }
    /// 负责生产消息
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + 'static + Send,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

/// 实现退出时需要的后续处理：
/// 1. 关闭发送端，使得连接断开，此时接收端会接收到错误
/// 2. 处理接收端的错误，让Worker离开loop
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in self.workers.drain(..) {
            worker.thread.join().unwrap();
            println!("Shutting down worker {}", worker.id);
        }
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // receiver.lock().unwrap() 得到mpsc::Receiver<Job>
                // 其若没有接收到消息则会阻塞
                let message = receiver.lock().unwrap().recv();
                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                };
            }
        });

        Worker { id, thread }
    }
}