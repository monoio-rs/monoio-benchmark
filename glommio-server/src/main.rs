use std::sync::Arc;

use config::{ServerConfig, PACKET_SIZE};
use futures_lite::{AsyncReadExt, AsyncWriteExt, StreamExt};
use glommio::{net::TcpListener, LocalExecutorBuilder, Placement};

fn main() {
    let cfg = Arc::new(ServerConfig::parse());
    println!(
        "Running ping pong server with Glommio.\nPacket size: {}\nListen {}\nCPU slot: {}",
        PACKET_SIZE,
        cfg.bind,
        config::format_cores(&cfg.cores)
    );

    let mut threads = Vec::new();
    for cpu in cfg.cores.iter() {
        let cfg_ = cfg.clone();
        let cpu_ = *cpu as _;
        let h = std::thread::spawn(move || {
            let ex = LocalExecutorBuilder::new(Placement::Fixed(cpu_))
                .make()
                .unwrap();

            ex.run(serve(cfg_));
        });
        threads.push(h);
    }
    for h in threads {
        let _ = h.join();
    }
}

async fn serve(cfg: Arc<ServerConfig>) {
    let listener = TcpListener::bind(&cfg.bind).unwrap();
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let mut stream = stream.unwrap();
        glommio::spawn_local(async move {
            let mut buf = vec![0; PACKET_SIZE];
            loop {
                match stream.read_exact(&mut buf).await {
                    Ok(_) => {}
                    Err(_) => return,
                }
                match stream.write_all(&buf).await {
                    Ok(_) => {}
                    Err(_) => return,
                }
            }
        })
        .detach();
    }
}
