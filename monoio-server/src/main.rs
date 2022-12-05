use std::sync::Arc;

use config::{ServerConfig, PACKET_SIZE};
use monoio::{net::TcpListener, RuntimeBuilder, io::{AsyncReadRentExt, AsyncWriteRentExt}};

fn main() {
    let cfg = Arc::new(ServerConfig::parse());
    println!(
        "Running ping pong server with Monoio.\nPacket size: {}\nListen {}\nCPU slot: {}",
        PACKET_SIZE,
        cfg.bind,
        config::format_cores(&cfg.cores)
    );

    let mut threads = Vec::new();
    for cpu in cfg.cores.iter() {
        let cfg_ = cfg.clone();
        let cpu_ = *cpu as _;
        let h = std::thread::spawn(move || {
            monoio::utils::bind_to_cpu_set(Some(cpu_)).unwrap();
            let mut rt = RuntimeBuilder::<monoio::IoUringDriver>::new()
                .with_entries(32768)
                .build()
                .unwrap();
            rt.block_on(serve(cfg_));
        });
        threads.push(h);
    }
    for h in threads {
        let _ = h.join();
    }
}

async fn serve(cfg: Arc<ServerConfig>) {
    let listener = TcpListener::bind(&cfg.bind).unwrap();
    while let Ok((mut stream, _)) = listener.accept().await {
        monoio::spawn(async move {
            let mut buf = vec![0; PACKET_SIZE];
            loop {
                let (r, buf_r) = stream.read_exact(buf).await;
                if r.is_err() {
                    // The connection is closed.
                    return;
                }
                let (w, buf_w) = stream.write_all(buf_r).await;
                if w.is_err() {
                    // The connection is closed.
                    return;
                }
                buf = buf_w;
            }
        });
    }
}
