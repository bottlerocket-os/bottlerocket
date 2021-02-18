#![deny(rust_2018_idioms)]

#[cfg(k8s_variant)]
mod static_pods;
#[cfg(k8s_variant)]
#[macro_use]
extern crate log;

#[cfg(k8s_variant)]
#[tokio::main]
async fn main() {
    static_pods::main().await
}

#[cfg(not(k8s_variant))]
fn main() {}
