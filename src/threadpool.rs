use crate::sysinfo::SystemObject;
use lazy_static::lazy_static;
use std::panic;
use std::process;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Barrier, Mutex};
use std::thread::{current, Builder, JoinHandle};

lazy_static! {
    static ref INSTANCE: Arc<Mutex<ThreadPoolManager>> =
        Arc::new(Mutex::new(ThreadPoolManager::new()));
}

pub type Job = Arc<dyn Fn() + Send + Sync>;

pub fn as_static_job<T>(capture: T) -> Job
where
    T: Fn() + Send + Sync + 'static,
{
    Arc::new(capture)
}

pub struct ThreadPoolManager {
    pub num_threads: usize,
    task_barrier: Arc<Barrier>,
    task_comms: Vec<Sender<Job>>,
    _thread_pool: Vec<JoinHandle<()>>,
}

impl ThreadPoolManager {
    fn new() -> ThreadPoolManager {
        let master_hook = panic::take_hook();
        // Crash the program if any of our threads panic
        panic::set_hook(Box::new(move |info| {
            // Only panic on our own threads, leave application programmer's threads alone
            if current().name().unwrap_or_default().starts_with("RMP_PAR_THREAD_") {
                master_hook(info);
                process::exit(1);
            } else {
                master_hook(info);
            }
        }));

        let num_threads = SystemObject::get_instance().max_num_threads;
        let task_barrier = Arc::new(Barrier::new(num_threads + 1));
        let mut _thread_pool = Vec::new();
        let mut task_comms = Vec::new();

        for tid in 0..num_threads {
            let task_barrier = task_barrier.clone();
            let builder = Builder::new() // Thread builder configuration
                .name(format!("RMP_PAR_THREAD_{}", tid)) // Name: RMP_PAR_THREAD_tid
                .stack_size(8 << 20); // Stack size: 8MB (Linux default)
            let (sender, receiver) = channel::<Job>();
            task_comms.push(sender);
            _thread_pool.push(
                builder
                    .spawn(move || routine_wrapper(tid, task_barrier, receiver))
                    .unwrap(),
            );
        }

        ThreadPoolManager {
            num_threads,
            task_barrier,
            task_comms,
            _thread_pool,
        }
    }

    pub fn get_instance_guard() -> Arc<Mutex<ThreadPoolManager>> {
        return INSTANCE.clone();
    }

    pub fn exec(&self, tasks: Vec<Job>) {
        // Used to wake up threads
        self.task_barrier.wait();
        assert_eq!(SystemObject::get_instance().max_num_threads, tasks.len());
        for i in 0..tasks.len() {
            self.task_comms[i].send(tasks[i].clone()).unwrap();
        }
        // Used to return main thread from exec
        self.task_barrier.wait();
    }
}

fn routine_wrapper(tid: usize, task_barrier: Arc<Barrier>, receiver: Receiver<Job>) {
    SystemObject::get_instance()
        .set_affinity(tid)
        .unwrap_or_else(|e| eprintln!("Failed to bind process #{} to hwthread: {:?}", tid, e));
    loop {
        task_barrier.wait();
        let func = receiver.recv().unwrap();
        func();
        task_barrier.wait();
    }
}
