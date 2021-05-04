use rand::random;
use rayon::prelude::*;
use std::cmp::max;
use std::env;
use std::time::Instant;

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
    let timer = Instant::now();
    let _result = (0..nsize)
        .into_par_iter()
        .map(|i| {
            let mut res_row = Vec::with_capacity(nsize);
            for j in 0..nsize {
                let mut sum: f64 = 0.0;
                for k in 0..nsize {
                    sum += matrix[i][k] * matrix[k][j];
                }
                res_row.push(sum);
            }
            res_row
        })
        .collect::<Vec<Vec<f64>>>();
    let interval = timer.elapsed();
    println!("Elapsed time: {:?}", interval);
}
