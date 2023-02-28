#[cfg(variant_runtime = "k8s")]
mod static_pods;
#[cfg(variant_runtime = "k8s")]
#[macro_use]
extern crate log;

#[cfg(variant_runtime = "k8s")]
#[tokio::main]
async fn main() {
    static_pods::main().await
}

#[cfg(not(variant_runtime = "k8s"))]
fn main() {}
