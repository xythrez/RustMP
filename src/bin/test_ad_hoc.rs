use std::sync::*;
//use std::sync::RwLock;
use std::sync::atomic::*;
use std::thread;
//use rustmp::rmp_parallel_for;

/*
 * A basic sequential function that we want to modify.
 * This is written in the most human readable way possible. Not compatable
 * with `#[rmp_parallel_for]`.
 */
fn seq_main() {
    let mut counter = 0;

    for i in 0..4 {
        println!("Index {}: Hello from loop {}!", counter, i);
        counter += 1;
    }
}

/*
 * What the user should write for an RustMP parallel program
 * The parallel section needs to be in its own function, due to current
 * Rust limitations with custom attributes on expressions.
 * See <https://github.com/rust-lang/rust/issues/54727> for details
 *
 * Function should be valid Rust regardless of whether macro is applied.
 * Whether Arc gets applied automatically or manually is up for debate.
 */
fn aug_main() {
    let mut counter = 0;

    //#[rmp_parallel_for(shared(counter) schedule(static, 1))]
    fn _loop(counter: &mut i32) {
    for i in 0..4 {
        println!("Index {}: Hello from loop {}!", counter, i);
        *counter += 1;
    }} _loop(&mut counter);
}

/*
 * What `#[rmp_parallel_for]` should convert the function into
 * Current implementation is still ad-hoc, but hopefully the
 * macro would expand the function as designed.
 *
 * Number of threads RMP_INTERNAL_MAX_THREADS = 4
 */
fn rmp_main() {
    let mut counter = 0;

    fn _loop(counter: &mut i32) {
    // Startup - Populate environment variables using env::var
    let RMP_INTERNAL_MAX_THREADS = 4;

    // Startup - Populate macro parameters
    let RMP_INTERNAL_BLOCK_SIZE = 1;

    // Startup - Initialize required arrays
    let mut RMP_INTERNAL_THREADS_ARR = vec![];
    let mut RMP_INTERNAL_ITER_ARR = vec![];
    for _ in 0..RMP_INTERNAL_MAX_THREADS {
        RMP_INTERNAL_ITER_ARR.push(vec![]);
    }
    let mut RMP_INTERNAL_CURR_BLOCK_SIZE = 0;
    let mut RMP_INTERNAL_CURR_BLOCK_THREAD = 0;

    // Startup - Promote shared mutables into Arc references
    // Idea - Possible optimization based on type? RwLock is expensive.
    let RMP_VAR_counter = Arc::new(AtomicI32::new(*counter));

    // Execution - Precompute the iterations for each loop
    // The 0..4 here should be parsed from the original tokens
    for RMP_INTERNAL_I in 0..4 {
        RMP_INTERNAL_ITER_ARR[RMP_INTERNAL_CURR_BLOCK_THREAD].push(RMP_INTERNAL_I);
        RMP_INTERNAL_CURR_BLOCK_SIZE += 1;
        if RMP_INTERNAL_CURR_BLOCK_SIZE >= RMP_INTERNAL_BLOCK_SIZE {
            RMP_INTERNAL_CURR_BLOCK_THREAD = (RMP_INTERNAL_CURR_BLOCK_THREAD + 1) % RMP_INTERNAL_MAX_THREADS;
        }
    }

    // Execution - Spawn threads with loop contents
    for RMP_INTERNAL_ITER in RMP_INTERNAL_ITER_ARR {
        // Clone used Arcs here
        let RMP_VAR_counter = Arc::clone(&RMP_VAR_counter);

        // Spawn threads
        RMP_INTERNAL_THREADS_ARR.push(thread::spawn(move || {
            for i in RMP_INTERNAL_ITER {
                // Having separate load and fetch_add should be a data race,
                // However, I believe OpenMP also treats it as a data race,
                // so its fine to have this issue
                // Need to implement #[rmp_critical] to update it correctly
                println!("Index {}: Hello from loop {}!", RMP_VAR_counter.load(Ordering::SeqCst), i);
                RMP_VAR_counter.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }

    // Cleanup - Wait for threads
    for RMP_INTERNAL_THREAD in RMP_INTERNAL_THREADS_ARR {
        let _ = RMP_INTERNAL_THREAD.join();
    }

    // Cleanup - Restore variables from Arc references
    *counter = RMP_VAR_counter.load(Ordering::SeqCst);
    } _loop(&mut counter);
}

/*
 * A basic parallel function written by hand in the most human readable way
 * possible.
 */
fn par_main() {
    let counter = Arc::new(AtomicIsize::new(0));
    let mut children = vec![];

    for i in 0..4 {
        let counter = Arc::clone(&counter);
        children.push(thread::spawn(move || {
            let index = counter.fetch_add(1, Ordering::SeqCst);
            println!("Index {}: Hello from loop {}!", index, i);
        }));
    }

    for child in children {
        let _ = child.join();
    }
}


fn main() {
    println!("Running Sequential Version:");
    seq_main();

    println!("\nRunning Augmented Sequential Version:");
    aug_main();

    println!("\nRunning Ad-hoc Parallel Version:");
    par_main();

    println!("\nRunning Augmented Parallel Version:");
    rmp_main();
}
