#![feature(test)]
extern crate test;

use num::{BigUint, One};
use rayon::prelude::*;
use rustmp::par_for;
use std::ops::Mul;

const N: u32 = 9999;

// following functions copied from rayon-demo

/// Compute the Factorial using a plain iterator.
fn factorial(n: u32) -> BigUint {
    (1..=n).map(BigUint::from).fold(BigUint::one(), Mul::mul)
}

#[bench]
/// Benchmark the Factorial using a plain iterator.
fn factorial_iterator(b: &mut test::Bencher) {
    let f = factorial(N);
    b.iter(|| assert_eq!(factorial(test::black_box(N)), f));
}

#[bench]
/// Compute the Factorial using rayon::par_iter.
fn factorial_par_iter(b: &mut test::Bencher) {
    fn fact(n: u32) -> BigUint {
        (1..n + 1)
            .into_par_iter()
            .map(BigUint::from)
            .reduce_with(Mul::mul)
            .unwrap()
    }

    let f = factorial(N);
    b.iter(|| assert_eq!(fact(test::black_box(N)), f));
}

// end functions copied from rayon-demo

#[bench]
/// Compute the Factorial using rustmp::par_for.
fn factorial_rmp(b: &mut test::Bencher) {
    fn fact(n: u32) -> BigUint {
        let mut res = BigUint::one();
        par_for! {
            for i in 2..n + 1, reduction res#*, {
                res *= BigUint::from(i);
            }
        }
        res
    }

    let f = factorial(N);
    b.iter(|| assert_eq!(fact(test::black_box(N)), f));
}
