/// Generators for updates settings.
use rand::{thread_rng, Rng};

pub fn generate_seed() -> u32 {
    let mut rng = thread_rng();
    rng.gen_range(0..2048)
}
