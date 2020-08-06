#[cfg(variant = "aws-ecs-1")]
mod ecs;

#[cfg(variant = "aws-ecs-1")]
fn main() {
    ecs::main()
}

#[cfg(not(variant = "aws-ecs-1"))]
fn main() {}
