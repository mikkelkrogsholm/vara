//! Offline local acceptance entry. Cloud evaluation requires the signed desktop harness.
#[tokio::main]
async fn main() {
    let path = std::env::args().nth(1).expect("plan path");
    vara_engine::acceptance::run(std::path::Path::new(&path), |engine| {
        if engine.locality == vara_core::Locality::Cloud {
            Err("signed desktop required".into())
        } else {
            Ok(None)
        }
    })
    .await
    .expect("acceptance run");
}
