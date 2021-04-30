// use rand::Rng;
use rustmp::{par_for, critical};
// use std::time;

#[derive(Debug)]
struct Student {
    name: String,
    age: u8,
    gpa: f32,
}

impl Student {
    pub fn new(age: u8) -> Student {
        Student {
            name: "Default".to_string(),
            age,
            gpa: age as f32,
        }
    }
}

fn min(x: i32, y: &i32) -> i32 {
    if x < *y {
        x
    } else {
        *y
    }
}

fn main() {
    let numbers: Vec<Student> = vec![];

    par_for! {
        for i in 1..32, blocksize 4, shared_mut numbers, {
            //std::thread::sleep(
            //    time::Duration::from_secs(
            //    rand::thread_rng().gen_range(1..10)));
            critical! {
                // Automatically locks numbers as read+write,
                // and makes the result accessible as number
                readwrite numbers;
                numbers.push(Student::new(i));
            }
            println!("Thread {} running!", i);
        }
    }

    for num in numbers {
        println!("{:?}", num);
    }

    let a = vec![vec![1,2,3],vec![4,5,6],vec![7,8,9]];
    let b = vec![vec![3,2,1],vec![6,5,4],vec![9,8,7]];
    let n = a.len();
    let mut c = vec![vec![0;n];n];
    for i in 0..n {
        for j in 0..n {
            let mut x = 0;
            par_for! {
                for k in 0..n, shared a b, reduction x#+, {
                    x += (a[i][k]*b[k][j]);
                }
            }
            c[i][j] = x;
        }
    }
    println!("{:?}", c);
    // let mut local = 0;
    // par_for! {
    // for i in 1..32, blocksize 1, shared_mut numbers, private local, {
    // local += 1;
    // println!("{}", local);
    // let mut lock = numbers.write();
    // lock.push(Student::new(i));
    // println!("Thread {} running!", i);
    // } }

}
