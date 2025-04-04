use std::sync::Arc;

use config::{PACKET_SIZE, ServerConfig};
use smol::future;
use smol::io::{AsyncReadExt, AsyncWriteExt};
use smol::{Executor, net::TcpListener};

fn main() {
    let cfg = Arc::new(ServerConfig::parse());
    let cores = cfg.cores.len();
    println!(
        "Running ping pong server with Smol.\nPacket size: {}\nListen {}\nCPU count: {}",
        PACKET_SIZE, cfg.bind, cores
    );

    let ex = Arc::new(smol::Executor::new());
    future::block_on(ex.run(serve(cfg, ex.clone())));
}

async fn serve(cfg: Arc<ServerConfig>, ex: Arc<Executor<'_>>) {
    let listener = TcpListener::bind(&cfg.bind).await.unwrap();

    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        ex.spawn(async move {
            let mut buf = vec![0; PACKET_SIZE];
            loop {
                match stream.read_exact(&mut buf).await {
                    Ok(_) => {}
                    Err(_) => {
                        return;
                    }
                }
                match stream.write_all(&buf).await {
                    Ok(_) => {}
                    Err(_) => {
                        return;
                    }
                }
            }
        })
        .detach();
    }
}
