// libraries used:
// std::sync::Arc;
// std::sync::RwLock;
// std::sync::atomic::AtomicIsize;
// std::sync::atomic::AtomicI32;
// std::sync::atomic::Ordering;
// std::thread;

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
        }
    }
    _loop(&mut counter);
}

/*
 * What `#[rmp_parallel_for]` should convert the function into
 * Current implementation is still ad-hoc, but hopefully the
 * macro would expand the function as designed.
 *
 * Number of threads __rmp_internal_max_threads = 4
 */
fn rmp_main() {
    let mut counter = 0;

    fn _loop(counter: &mut i32) {
        // Startup - Populate environment variables using env::var
        let __rmp_internal_max_threads = 4;

        // Startup - Populate macro parameters
        let __rmp_internal_block_size = 1;

        // Startup - Initialize required arrays
        let mut __rmp_internal_threads_arr = vec![];
        let mut __rmp_internal_iter_arr = vec![];
        for _ in 0..__rmp_internal_max_threads {
            __rmp_internal_iter_arr.push(vec![]);
        }
        let mut __rmp_internal_curr_block_size = 0;
        let mut __rmp_internal_curr_block_thread = 0;

        // Startup - Promote shared mutables into Arc references
        // Idea - Possible optimization based on type? RwLock is expensive.
        let __rmp_var_counter = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(*counter));

        // Execution - Precompute the iterations for each loop
        // The 0..4 here should be parsed from the original tokens
        for __rmp_internal_i in 0..4 {
            __rmp_internal_iter_arr[__rmp_internal_curr_block_thread].push(__rmp_internal_i);
            __rmp_internal_curr_block_size += 1;
            if __rmp_internal_curr_block_size >= __rmp_internal_block_size {
                __rmp_internal_curr_block_thread =
                    (__rmp_internal_curr_block_thread + 1) % __rmp_internal_max_threads;
            }
        }

        // Startup - Extract the thread's own iterator
        let __rmp_internal_iter_self = __rmp_internal_iter_arr.remove(0);

        // Execution - Spawn threads with loop contents
        for __rmp_internal_iter in __rmp_internal_iter_arr {
            // Clone used Arcs here
            let __rmp_var_counter = std::sync::Arc::clone(&__rmp_var_counter);

            // Spawn threads
            __rmp_internal_threads_arr.push(std::thread::spawn(move || {
                for i in __rmp_internal_iter {
                    // Having separate load and fetch_add should be a data race,
                    // However, I believe OpenMP also treats it as a data race,
                    // so its fine to have this issue
                    // Need to implement #[rmp_critical] to update it correctly
                    println!(
                        "Index {}: Hello from loop {}!",
                        __rmp_var_counter.load(std::sync::atomic::Ordering::SeqCst),
                        i
                    );
                    __rmp_var_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
            }));
        }

        // Execution - Extract the same thread logic for self
        for i in __rmp_internal_iter_self {
            println!(
                "Index {}: Hello from loop {}!",
                __rmp_var_counter.load(std::sync::atomic::Ordering::SeqCst),
                i
            );
            __rmp_var_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }

        // Cleanup - Wait for threads
        for __rmp_internal_thread in __rmp_internal_threads_arr {
            let _ = __rmp_internal_thread.join();
        }

        // Cleanup - Restore variables from Arc references
        *counter = __rmp_var_counter.load(std::sync::atomic::Ordering::SeqCst);
    }
    _loop(&mut counter);
}

/*
 * A basic parallel function written by hand in the most human readable way
 * possible.
 */
fn par_main() {
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicIsize::new(0));
    let mut children = vec![];

    for i in 1..4 {
        let counter = std::sync::Arc::clone(&counter);
        children.push(std::thread::spawn(move || {
            let index = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            println!("Index {}: Hello from loop {}!", index, i);
        }));
    }
    let index = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    println!("Index {}: Hello from loop {}!", index, 0);

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
