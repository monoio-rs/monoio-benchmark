use config::{ServerConfig, PACKET_SIZE};

fn main() {
    let cfg = ServerConfig::parse();
    println!(
        "Running ping pong server with Tokio.\nPacket size: {}\nListen {}",
        PACKET_SIZE, cfg.bind
    );

    tokio_uring::start(async {
        serve(&cfg).await;
    });
}

fn successful(result: Result<usize, std::io::Error>) -> bool {
    if let Ok(size) = result {
        return size == PACKET_SIZE;
    }

    false
}

async fn serve(cfg: &ServerConfig) {
    let listener = tokio_uring::net::TcpListener::bind(cfg.bind.parse().unwrap()).unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();

        tokio_uring::spawn(async move {
            let mut buf = vec![0; PACKET_SIZE];

            loop {
                let (result, buf2) = stream.read(buf).await;

                if !successful(result) {
                    return;
                }

                buf = buf2;

                let (result, buf2) = stream.write(buf).await;

                if !successful(result) {
                    return;
                }

                buf = buf2;
            }
        });
    }
}
