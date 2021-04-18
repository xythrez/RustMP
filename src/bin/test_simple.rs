use rand::Rng;
use rustmp::par_for;
use std::time;

#[derive(Debug)]
struct Student {
    name: String,
    age: u8,
    gpa: f32,
}

impl Student {
    pub fn new(age: u8) -> Student {
    Student { name: "Default".to_string(),
          age: age,
          gpa: age as f32 }
    }
}

fn main() {
    let numbers: Vec<Student> = vec![];

    par_for! {
    for i in 1..10, capturing numbers {

    //std::thread::sleep(
    //    time::Duration::from_secs(
    //    rand::thread_rng().gen_range(1..10)));
    let mut lock = numbers.write();
    lock.push(Student::new(i));
    println!("Thread {} running!", i);
    } };

    for num in numbers {
    println!("{:?}", num);
    }
}
