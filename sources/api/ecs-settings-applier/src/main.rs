#[cfg(variant = "aws-ecs-1")]
mod ecs;

#[cfg(variant = "aws-ecs-1")]
#[tokio::main]
async fn main() {
    ecs::main().await
}

#[cfg(not(variant = "aws-ecs-1"))]
fn main() {}
