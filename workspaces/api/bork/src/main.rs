use rand::{thread_rng, Rng};

fn main() {
    let mut rng = thread_rng();
    println!("{}", rng.gen_range(0, 2048));
}
