use std::sync::Arc;

use config::{PACKET_SIZE, ServerConfig};
use socket2::{Protocol, Type};

fn main() {
    let cfg = Arc::new(ServerConfig::parse());
    println!(
        "Running ping pong server with Tokio Uring.\nPacket size: {}\nListen {}",
        PACKET_SIZE, cfg.bind
    );

    if cfg.cores.len() == 1 {
        tokio_uring::start(async {
            serve(&cfg).await;
        });
    } else {
        let mut threads = Vec::with_capacity(cfg.cores.len());
        for cpu in cfg.cores.iter() {
            let cfg_ = cfg.clone();
            let cpu_ = *cpu as _;
            let h = std::thread::spawn(move || {
                let mut cpu_set = nix::sched::CpuSet::new();
                cpu_set.set(cpu_).unwrap();
                nix::sched::sched_setaffinity(nix::unistd::Pid::from_raw(0), &cpu_set).unwrap();

                let mut builder = tokio_uring::builder();
                builder.entries(32768);
                let rt = tokio_uring::Runtime::new(&builder).unwrap();
                rt.block_on(serve(&cfg_));
            });
            threads.push(h);
        }

        for thread in threads {
            thread.join().unwrap();
        }
    }
}

async fn serve(cfg: &ServerConfig) {
    let addr = cfg.bind.parse().unwrap();
    let domain = socket2::Domain::for_address(addr);

    let socket = socket2::Socket::new(domain, Type::STREAM, Some(Protocol::TCP)).unwrap();
    socket.set_reuse_port(true).unwrap(); // this is not done by default like it is on monoio
    socket.bind(&addr.into()).unwrap();
    socket.listen(1024).unwrap();

    let listener: std::net::TcpListener = socket.into();
    let listener = tokio_uring::net::TcpListener::from_std(listener);

    loop {
        let (stream, _) = listener.accept().await.unwrap();

        tokio_uring::spawn(async move {
            let mut buf = vec![0; PACKET_SIZE];

            loop {
                let (result, buf2) = stream.read(buf).await;

                if result.is_err() {
                    return;
                }

                buf = buf2;

                let (result, buf2) = stream.write(buf).submit().await;

                if result.is_err() {
                    return;
                }

                buf = buf2;
            }
        });
    }
}
