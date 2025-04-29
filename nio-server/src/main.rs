use config::{PACKET_SIZE, ServerConfig};
use nio::{net::TcpListener, runtime::Runtime};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn main() {
    let cfg = ServerConfig::parse();
    let cores = cfg.cores.len();
    println!(
        "Running ping pong server with Nio.\nPacket size: {}\nListen {}\nCPU count: {}",
        PACKET_SIZE, cfg.bind, cores
    );

    let rt = nio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(cores)
        .build()
        .unwrap();

    rt.block_on(serve(&cfg, &rt))
}

async fn serve(cfg: &ServerConfig, rt: &Runtime) {
    let listener = TcpListener::bind(&cfg.bind).await.unwrap();

    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        rt.spawn(async move {
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
        });
    }
}
