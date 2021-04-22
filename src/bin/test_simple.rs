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
fn main() {
    let numbers: Vec<Student> = vec![];

    par_for! {
        for i in 1..32, blocksize 4, capturing numbers, {
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

    let a:Vec<Vec<i32>> = vec![vec![1,2,3],vec![4,5,6],vec![7,8,9]];
    let n = a.len();
    let mut c = vec![vec![0;n];n];
    let mut x = 0;
    par_for! {
        // I feel like we shouldn't need to capture a here
        for k in 0..n, capturing a, reduction x#+, {
            critical! {
                read a;
                x += a[1][k]*a[k][0];
            }
        }
    }
    c[1][0] = x;
    println!("{:?}", c[1][0]);
    // let mut local = 0;
    // par_for! {
    // for i in 1..32, blocksize 1, capturing numbers, private local, {
    // local += 1;
    // println!("{}", local);
    // let mut lock = numbers.write();
    // lock.push(Student::new(i));
    // println!("Thread {} running!", i);
    // } }

}
