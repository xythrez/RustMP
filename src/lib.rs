pub mod threadpool;

mod sysinfo;

use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub use threadpool::{as_static_job, Job, ThreadPoolManager};

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
macro_rules! __internal_par_for {
    ($name:ident, $iterator:expr, $size:expr, $($captured:ident)*, $blk:block) => {
        let mut __rmp_tasks = Vec::new();
        $(let $captured = rustmp::Capture::new($captured);)*
        {
            let __rmp_tpm_mtx = rustmp::ThreadPoolManager::get_instance_guard();
            let __rmp_tpm = __rmp_tpm_mtx.lock().unwrap();
            let __rmp_iters = __rmp_tpm.split_iterators($iterator, $size);
            for iter in __rmp_iters {
                $(let $captured = $captured.clone();)*
                __rmp_tasks.push(rustmp::as_static_job(move || {
                    for &$name in &iter
                        $blk
                }));
            }
            __rmp_tpm.exec(__rmp_tasks);
        }
        $(let $captured = $captured.unwrap();)*
    };
}

/// "parallel for" wrapper
///
/// If the number of arguments increases, convert this to a tail recursive parser instead.
/// Current implementation save limited (max depth 32) stack space for macro expansion.
#[macro_export]
macro_rules! par_for {
    (for $name:ident in $iterator:expr, blocksize $size:expr, capturing $($captured:ident)+, $blk:block) => {
        rustmp::__internal_par_for!($name, $iterator, $size, $($captured)*, $blk);
    };
    (for $name:ident in $iterator:expr, blocksize $size:expr, capturing $($captured:ident)+ $blk:block) => {
        rustmp::__internal_par_for!($name, $iterator, $size, $($captured)*, $blk);
    };
    (for $name:ident in $iterator:expr, capturing $($captured:ident)+, blocksize $size:expr, $blk:block) => {
        rustmp::__internal_par_for!($name, $iterator, $size, $($captured)*, $blk);
    };
    (for $name:ident in $iterator:expr, blocksize $size:expr, $blk:block) => {
        rustmp::__internal_par_for!($name, $iterator, $size,, $blk);
    };
    (for $name:ident in $iterator:expr, capturing $($captured:ident)+, $blk:block) => {
        rustmp::__internal_par_for!($name, $iterator, 1, $($captured)*, $blk);
    };
    (for $name:ident in $iterator:expr, capturing $($captured:ident)+ $blk:block) => {
        rustmp::__internal_par_for!($name, $iterator, 1, $($captured)*, $blk);
    }
}
