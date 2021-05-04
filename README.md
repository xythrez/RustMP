RustMP
======

This repository contains the source code for Alex Bowman, Paul Ouellette, Raffi
Sanna, and Jack Yu's research project for CSC200H: an implicit parallelism
library written in Rust. For more details, please see the project report.

## Authors
- Alex Bowman (abowman6@u.rochester.edu)
- Paul Ouellette (pouellet@u.rochester.edu)
- Raffi Sanna (rsanna@u.rochester.edu)
- Jack Yu (yyu57@u.rochester.edu)

## Building/Running

RustMP requires the following dependencies to run:
- `hwloc2-v2.2.0`: C HWLoc wrapper used for NUMA aware process pinning
- `lazy_static-v1.4.0`: Lazy static macro used for delayed singleton init
- `num-v0.3`: Needed to run the Rust factorial benchmarks
- `rand-v0.8.3`: Random number generation used for benchmarking examples
- `rayon-v1.5.0`: Rayon parallel iterator library used in RustMP/Rayon..
comparision tests

`src/` contains all code for the RustMP library, including `lib.rs`,
`sysinfo.rs`, and `threadpool.rs`. `src/bin` contains benchmarking programs
demonstrated in our paper. To run one of the benchmarks with cargo, execute the
following command:

```
# Set the RustMP, OpenMP, and Rayon thread pool thread count to <n>
$ export RMP_NUM_THREADS=<n>
$ export OMP_NUM_THREADS=<n>
$ export RAYON_NUM_THREADS=<n>

# Run the test
$ cargo run --release --bin <testname>
```

C comparison benchmarks can be found in `omp/`. These can be compiled using
`make`. All matrix multiplication tests support one integer input as for the
square matrix dimensions (i.e. `./matmul <nsize>`).

Additional benchmarks are located in `benches`. These require Rust nightly and
can be run with the following command:

```
$ cargo +nightly bench
```

## Known issues

Due to Rust compiler limitations, Rust is only able to support up to 32 layers
of nested macros by default. If an alternative test program that exceeds this
nested depth for macros is used, please consider increasing this limit in
`Cargo.toml`.

