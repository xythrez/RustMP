use rustmp::rmp_parallel_for;

fn main() {
    #[rmp_parallel_for]
    fn inner() {
        for i in 1..10 {
            println!("Hello from {}!", i);
        }
    }

    inner();
}
