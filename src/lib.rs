pub mod threadpool;

mod sysinfo;

use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct Capture<T> {
    value: Arc<RwLock<T>>,
}

impl<T> Capture<T> {
    pub fn new(inner: T) -> Capture<T> {
        return Capture {
            value: Arc::new(RwLock::new(inner)),
        };
    }

    pub fn clone(&self) -> Capture<T> {
        Capture {
            value: Arc::clone(&self.value),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<T> {
        return self.value.as_ref().read().unwrap();
    }

    pub fn write(&self) -> RwLockWriteGuard<T> {
        return self.value.as_ref().write().unwrap();
    }

    pub fn unwrap(self) -> T {
        Arc::try_unwrap(self.value)
            .ok()
            .and_then(|o| o.into_inner().ok())
            .expect("Error: reference copied out of loop")
    }
}

#[macro_export]
macro_rules! par_for {
    (for $name:ident in $iterator:expr, capturing $captured:ident $blk:block) => {
        use rustmp::Capture;
        use rustmp::threadpool::{Job, ThreadPoolManager, as_static_job};
        use std::sync::{Arc, RwLock};
        use std::thread;

        let mut tasks = Vec::new();
        let $captured = Capture::new($captured);
        {
            let tpm_mtx = ThreadPoolManager::get_instance_guard();
            let tpm = tpm_mtx.lock().unwrap();
            let iters = tpm.split_iterators($iterator, 1);
            for iter in iters {
                let $captured = $captured.clone();
                tasks.push(as_static_job(move || {
                    for &$name in &iter
                        $blk
                }));
            }
            tpm.exec(tasks);
        }

        //let itr = $iterator;
        //let $captured = Capture::new($captured);
        //let mut handles: Vec<thread::JoinHandle<()>> = vec![];
        //for $name in itr {
        //    let $captured = $captured.clone();
        //    handles.push(thread::spawn(move || $blk));
        //}

        //for handle in handles {
        //    handle.join().expect("Thread paniced!");
        //}

        let $captured = $captured.unwrap();
    };
}
