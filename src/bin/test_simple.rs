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

    let mut x = 0;
    // let mut y = 1;
    par_for! {
        for i in 0..10, reduction x#+, {
            // let mut lock = x.write();
            // *lock += 7;
            // y *= 6;
            x += 2;
        }
    }
    println!("{:?}", x);

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
