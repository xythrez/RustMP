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
macro_rules! critical {
    (read $($r:ident)+; readwrite $($w:ident)+; $($ops:tt)+) => {
        {
            $(let $r = $r.read();)*
            $(let mut $w = $w.write();)*
            $($ops)*
        }
    };
    (readwrite $($w:ident)+; read $($r:ident)+; $($ops:tt)+) => {
        {
            $(let $r = $r.read();)*
            $(let mut $w = $w.write();)*
            $($ops)*
        }
    };
    (readwrite $($w:ident)+; $($ops:tt)+) => {
        {
            $(let mut $w = $w.write();)*
            $($ops)*
        }
    };
    (read $($r:ident)+; $($ops:tt)+) => {
        {
            $(let $r = $r.read();)*
            $($ops)*
        }
    };
}

#[macro_export]
macro_rules! __internal_par_for {
    // without reduction
    (var_name($name:ident),
    iterator($iter:expr),
    blocksize($size:expr),
    captured($($captured:ident)*),
    private($($private:ident)*),
    reduction(),
    $blk:block) => {
        let mut __rmp_tasks = Vec::new();
        $(let $captured = rustmp::Capture::new($captured);)*
        {
            let __rmp_tpm_mtx = rustmp::ThreadPoolManager::get_instance_guard();
            let __rmp_tpm = __rmp_tpm_mtx.lock().unwrap();
            let __rmp_iters = __rmp_tpm.split_iterators($iter, $size);
            for iter in __rmp_iters {
                $(let $captured = $captured.clone();)*
                $(let $private = $private.clone();)*
                __rmp_tasks.push(rustmp::as_static_job(move || {
                    $(let mut $private = $private.clone();)*
                    for &$name in &iter
                        $blk
                }));
            }
            __rmp_tpm.exec(__rmp_tasks);
        }
        $(let $captured = $captured.unwrap();)*
    };
    // with reduction
    (var_name($name:ident),
    iterator($iter:expr),
    blocksize($size:expr),
    captured($($captured:ident)*),
    private($($private:ident)*),
    reduction($($red_name:ident, $red_op:tt)+),
    $blk:block) => {
        let mut __rmp_tasks = Vec::new();
        $(let $captured = rustmp::Capture::new($captured);)*
        {
            let __rmp_tpm_mtx = rustmp::ThreadPoolManager::get_instance_guard();
            let __rmp_tpm = __rmp_tpm_mtx.lock().unwrap();
            let __rmp_iters = __rmp_tpm.split_iterators($iter, $size);
            let mut __rmp_red_vals = Vec::new();
            // let mut __rmp_count = 0;
            $(__rmp_red_vals.push(Vec::new()); $red_name = $red_name;)*
            let __rmp_red_vals = rustmp::Capture::new(__rmp_red_vals);
            for iter in __rmp_iters {
                $(let $captured = $captured.clone();)*
                let __rmp_red_vals = __rmp_red_vals.clone();
                $(let $private = $private.clone();)*
                $(let $red_name = $red_name.clone();)*
                __rmp_tasks.push(rustmp::as_static_job(move || {
                    $(let mut $private = $private.clone();)*
                    $(let mut $red_name = $red_name.clone();)*
                    for &$name in &iter
                        $blk
                    let mut __rmp_temp = __rmp_red_vals.write();
                    let mut __rmp_counter = 0;
                    $(__rmp_temp[__rmp_counter].push($red_name); __rmp_counter += 1;)*
                }));
            }
            __rmp_tpm.exec(__rmp_tasks);
            let mut __rmp_temp = __rmp_red_vals.read();
            let mut __rmp_counter = 0;
            $($red_name = __rmp_temp[__rmp_counter].iter().fold($red_name, |x, &y| {x $red_op y}); __rmp_counter += 1;)*;
        }
        $(let $captured = $captured.unwrap();)*
    };
    // Parse blocksize
    (var_name($name:ident),
    iterator($iter:expr),
    blocksize($size:expr),
    captured($($captured:ident)*),
    private($($private:ident)*),
    reduction($($red_name:ident, $red_op:tt)*),
    blocksize $new_size:expr,
    $($rem:tt)+) => {
        rustmp::__internal_par_for!(
            var_name($name),
            iterator($iter),
            blocksize($new_size),
            captured($($captured)*),
            private($($private)*),
            reduction($($red_name, $red_op)*),
            $($rem)*)
    };
    // Parse capturing
    (var_name($name:ident),
    iterator($iter:expr),
    blocksize($size:expr),
    captured($($captured:ident)*),
    private($($private:ident)*),
    reduction($($red_name:ident, $red_op:tt)*),
    capturing $($new_captured:ident)*,
    $($rem:tt)+) => {
        rustmp::__internal_par_for!(
            var_name($name),
            iterator($iter),
            blocksize($size),
            captured($($new_captured)*),
            private($($private)*),
            reduction($($red_name, $red_op)*),
            $($rem)*)
    };
    // Parse private
    (var_name($name:ident),
    iterator($iter:expr),
    blocksize($size:expr),
    captured($($captured:ident)*),
    private($($private:ident)*),
    reduction($($red_name:ident, $red_op:tt)*),
    private $($new_private:ident)*,
    $($rem:tt)+) => {
        rustmp::__internal_par_for!(
            var_name($name),
            iterator($iter),
            blocksize($size),
            captured($($captured)*),
            private($($new_private)*),
            reduction($($red_name, $red_op)*),
            $($rem)*)
    };
    // Parse reduction
    (var_name($name:ident),
    iterator($iter:expr),
    blocksize($size:expr),
    captured($($captured:ident)*),
    private($($private:ident)*),
    reduction($($red_name:ident, $red_op:tt)*),
    reduction $($new_name:ident#$new_op:tt);*,
    $($rem:tt)+) => {
        rustmp::__internal_par_for!(
            var_name($name),
            iterator($iter),
            blocksize($size),
            captured($($captured)*),
            private($($private)*),
            reduction($($new_name, $new_op)*),
            $($rem)*)
    };
}

/// "parallel for" wrapper
///
/// If the number of arguments increases, convert this to a tail recursive parser instead.
/// Current implementation save limited (max depth 32) stack space for macro expansion.
#[macro_export]
macro_rules! par_for {
    (for $name:ident in $iter:expr, $($rem:tt)+) => {
        rustmp::__internal_par_for!(
            var_name($name),
            iterator($iter),
            blocksize(1),
            captured(),
            private(),
            reduction(),
            $($rem)*)
    }
}
