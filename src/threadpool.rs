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

/// The Job type used to submit tasks for the ThreadPoolManager
///
/// Most function captures can be cast to a Job directly. Or the
/// "as_static_job()" function can be used.
pub type Job = Arc<dyn Fn() + Send + Sync>;

/// Converts a function capture into a Job with a static lifetime.
///
/// It's also possible to use "Arc::new(|| {})" and cast it as a Job instead.
pub fn as_static_job<T>(capture: T) -> Job
where
    T: Fn() + Send + Sync + 'static,
{
    Arc::new(capture)
}

/// The ThreadPoolManager handles dispatching threads and sending Jobs to threads.
///
/// Only one thread can submit and execute Jobs to the ThreadPoolManager instance at a time.
/// Other threads attempting to lock the manager would wait on the ThreadPoolManager until
/// the last thread using it unlocks the instance.
pub struct ThreadPoolManager {
    pub num_threads: usize,
    task_barrier: Arc<Barrier>,
    task_comms: Vec<Sender<Job>>,
    _thread_pool: Vec<JoinHandle<()>>,
}

impl ThreadPoolManager {
    /// Creates a new ThreadPoolManager object.
    ///
    /// To get the current ThreadPoolManager, use get_instance_guard() instead.
    ///
    /// Should only be called by the INSTANCE.
    fn new() -> ThreadPoolManager {
        let master_hook = panic::take_hook();
        // Crash the program if any of our threads panic
        panic::set_hook(Box::new(move |info| {
            // Only panic on our own threads, leave application programmer's threads alone
            if current()
                .name()
                .unwrap_or_default()
                .starts_with("RMP_PAR_THREAD_#")
            {
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
                .name(format!("RMP_PAR_THREAD_#{}", tid)) // Name: RMP_PAR_THREAD_tid
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

    /// Gets the current ThreadPoolManager instance.
    ///
    /// The instance needs to be locked before using, not unlocking the TPM after use
    /// may result in deadlock.
    pub fn get_instance_guard() -> Arc<Mutex<ThreadPoolManager>> {
        return INSTANCE.clone();
    }

    /// Execute a set of tasks on the ThreadPoolManager.
    ///
    /// The task vector must be the same size as the number of threads, otherwise a panic will
    /// be thrown.
    pub fn exec(&self, tasks: Vec<Job>) {
        // Used to wake up threads
        self.task_barrier.wait();
        assert_eq!(self.num_threads, tasks.len());
        for i in 0..tasks.len() {
            self.task_comms[i].send(tasks[i].clone()).unwrap();
        }
        // Used to return main thread from exec
        self.task_barrier.wait();
    }

    /// Splits an iterator into RMP_NUM_THREADS iterators, each with a step size of
    /// block_size.
    ///
    /// Returned iterators are stored in a Vec<Vec<S>>, but anything should work as
    /// long as the default Rust for loop accepts it.
    pub fn split_iterators<T, S>(&self, iter: T, block_size: usize) -> Vec<Vec<S>>
    where
        T: Iterator<Item = S>,
    {
        let mut split = Vec::new();
        split.reserve_exact(self.num_threads);
        for _ in 0..self.num_threads {
            split.push(Vec::new());
        }

        let mut index: usize = 0;
        let mut block: usize = 0;
        for element in iter {
            split[index].push(element);
            block += 1;
            if block % block_size == 0 {
                block = 0;
                index = (index + 1) % self.num_threads;
            }
        }
        split
    }
}

/// Wrapper routine for threads in the ThreadPoolManager
fn routine_wrapper(tid: usize, task_barrier: Arc<Barrier>, receiver: Receiver<Job>) {
    SystemObject::get_instance()
        .set_affinity(tid)
        .unwrap_or_else(|e| eprintln!("Failed to bind process #{} to hwthread: {:?}", tid, e));
    loop {
        task_barrier.wait();
        receiver.recv().unwrap()();
        task_barrier.wait();
    }
}
