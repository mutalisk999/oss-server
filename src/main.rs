use dotenv::dotenv;
use flexi_logger::{detailed_format, Duplicate};
use log::info;
use std::net::SocketAddr;
use tokio::signal;

use crate::router::register_router;

mod controller;
mod router;

fn init_log() {
    flexi_logger::Logger::with_str("debug")
        .log_to_file()
        .directory("log")
        .basename("oss-server.log")
        .duplicate_to_stdout(Duplicate::All)
        .format_for_files(detailed_format)
        .format_for_stdout(detailed_format)
        .start()
        .unwrap_or_else(|e| panic!("logger initialization failed, err: {}", e));
}

async fn shutdown_signal() {
    #[cfg(unix)]
    let ctrl_c = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install Ctrl+C handler")
            .recv()
            .await;
        info!("terminated by SIGINT");
    };

    #[cfg(not(unix))]
    let ctrl_c = async {
        signal::windows::ctrl_c()
            .unwrap()
            .recv()
            .await
            .expect("failed to install Ctrl+C handler");
        info!("terminated by Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
        info!("terminated by SIGTERM");
    };

    #[cfg(not(unix))]
    let terminate = async {
        signal::windows::ctrl_break()
            .unwrap()
            .recv()
            .await
            .expect("failed to install Ctrl+Break handler");
        info!("terminated by Ctrl+Break");
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[tokio::main]
async fn main() {
    // init log
    init_log();

    dotenv().ok();

    // run it
    let listen_addr_str = "0.0.0.0:3000";
    let listen_addr: SocketAddr = listen_addr_str.parse().unwrap();

    let router = register_router();

    info!("listening on {}", listen_addr);
    axum::Server::bind(&listen_addr)
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
