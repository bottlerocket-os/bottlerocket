#[cfg(variant_family = "aws-ecs")]
mod ecs;

#[cfg(variant_family = "aws-ecs")]
#[tokio::main]
async fn main() {
    ecs::main().await
}

#[cfg(not(variant_family = "aws-ecs"))]
fn main() {}
