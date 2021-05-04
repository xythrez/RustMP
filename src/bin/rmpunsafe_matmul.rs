use rand::random;
use std::cmp::max;
use std::env;
use std::time::Instant;

use rustmp::par_for;

fn gen_matrix(nsize: usize) -> Vec<Vec<f64>> {
    let mut ret = Vec::with_capacity(nsize);
    for _ in 0..nsize {
        let mut row = Vec::with_capacity(nsize);
        for _ in 0..nsize {
            row.push((random::<f64>() - 0.5) * 255.0);
        }
        ret.push(row);
    }
    ret
}

fn gen_empty(nsize: usize) -> Vec<Vec<f64>> {
    let mut ret = Vec::with_capacity(nsize);
    let mut row = Vec::with_capacity(nsize);
    for _ in 0..nsize {
        row.push(0 as f64);
    }
    for _ in 0..nsize {
        ret.push(row.clone());
    }
    ret
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <msize>", args[0]);
        return;
    }
    let nsize = max(
        args[1].parse::<usize>().expect("Usage: matrix_mul <msize>"),
        1,
    );
    let matrix = gen_matrix(nsize);
    let mut result = gen_empty(nsize);
    let timer = Instant::now();
    par_for! {
        for i in 0..nsize, shared_unsafe result, shared matrix, {
            for j in 0..nsize {
                let mut sum = 0.0;
                for k in 0..nsize {
                    sum += matrix[i][k] * matrix[k][j];
                }
                result[i][j] = sum;
            }
        }
    }
    let interval = timer.elapsed();
    println!("Elapsed time: {:?}", interval);
}
