use std::sync::Arc;

use compio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    runtime::Runtime,
};
use config::{PACKET_SIZE, ServerConfig};
use socket2::{Protocol, Type};

fn main() {
    let cfg = Arc::new(ServerConfig::parse());
    println!(
        "Running ping pong server with Compio.\nPacket size: {}\nListen {}\nCPU slot: {}",
        PACKET_SIZE,
        cfg.bind,
        config::format_cores(&cfg.cores)
    );

    let mut threads = Vec::with_capacity(cfg.cores.len());
    for cpu in cfg.cores.iter() {
        let cfg_ = cfg.clone();
        let cpu_ = *cpu as _;
        let h = std::thread::spawn(move || {
            let mut cpu_set = nix::sched::CpuSet::new();
            cpu_set.set(cpu_).unwrap();
            nix::sched::sched_setaffinity(nix::unistd::Pid::from_raw(0), &cpu_set).unwrap();

            let rt = Runtime::new().unwrap();
            rt.block_on(serve(&cfg_));
        });
        threads.push(h);
    }

    for thread in threads {
        thread.join().unwrap();
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
    let listener = TcpListener::from_std(listener).unwrap();

    loop {
        let (mut stream, _) = listener.accept().await.unwrap();

        compio::runtime::spawn(async move {
            let mut buf = vec![0; PACKET_SIZE];
            loop {
                let res_r = stream.read_exact(buf).await;
                if res_r.is_err() {
                    return;
                }

                let buf_r = res_r.1;
                let res_w = stream.write_all(buf_r).await;
                if res_w.is_err() {
                    return;
                }
                buf = res_w.1;
            }
        })
        .detach();
    }
}
