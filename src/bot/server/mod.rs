use std::net::SocketAddr;
use std::sync::Arc;
use axum::body::Body;
use axum::{Router, routing::get, serve};
use axum::response::Response;
use logfather::info;
use sysinfo::Pid;
use tokio::net::TcpListener;
use crate::bot::core::structs::MetricsManager;

pub async fn run_metrics_manager(metrics_manager: Arc<MetricsManager>, pid: u32) {
    let updater = metrics_manager.clone();
    tokio::spawn(async move {
        let mut system = sysinfo::System::new_all();
        loop {
            updater.system_metrics.update(&mut system, Pid::from(pid as usize));
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 4020));
    info!("Metrics server listening on {}", addr);
    let router = Router::new()
        .route("/metrics", get(move || {
            let metrics_manager = metrics_manager.clone();
            async move {
                let metrics = metrics_manager.system_metrics.render_metrics();
                Response::builder()
                    .header("Content-Type", "text/plain; charset=utf-8; version=0.0.4")
                    .body(Body::from(metrics))
                    .unwrap()
            }
        }));

    let listener = TcpListener::bind(addr).await.unwrap();

    serve(listener, router).await.unwrap();
}