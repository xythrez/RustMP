use rustmp::threadpool::{ThreadPoolManager, Job, as_static_job};
use std::sync::Arc;

fn main() {
    let tpm_mtx= ThreadPoolManager::get_instance_guard();
    let tpm = tpm_mtx.lock().unwrap();

    println!("Submitting jobs!");
    let mut vector = Vec::new();
    for i in 0..tpm.num_threads {
        let cl = as_static_job(move || {println!("Hello from {}!", i)});
        vector.push(cl);
    }
    tpm.exec(vector);

    println!("Submitting more jobs with panic on tid=3!");
    let mut vector2 = Vec::new();
    for i in 0..tpm.num_threads {
        let x = 9;
        let cl = Arc::new(move || {
            if x * i == 27 {
                //panic!("Panic test");
            }
        }) as Job;
        vector2.push(cl);
    }
    tpm.exec(vector2);
}