use std::{
    io::{Read, Write},
    sync::Arc,
};

use config::{PACKET_SIZE, ServerConfig};

#[macro_use]
extern crate may;

use may::net::TcpListener;

fn main() {
    let cfg = Arc::new(ServerConfig::parse());
    let cores = cfg.cores.len();
    println!(
        "Running ping pong server with May.\nPacket size: {}\nListen {}\nCPU count: {}",
        PACKET_SIZE, cfg.bind, cores
    );

    may::config().set_workers(cores);

    may::coroutine::scope(|s| {
        for _ in 0..cores {
            let cfg = cfg.clone();
            go!(s, move || {
                let listener = TcpListener::bind(&cfg.bind).unwrap();

                loop {
                    let (mut stream, _) = listener.accept().unwrap();
                    go!(move || {
                        let mut buf = vec![0; PACKET_SIZE];
                        loop {
                            match stream.read_exact(&mut buf) {
                                Ok(_) => {}
                                Err(_) => {
                                    return;
                                }
                            }
                            match stream.write_all(&buf) {
                                Ok(_) => {}
                                Err(_) => {
                                    return;
                                }
                            }
                        }
                    });
                }
            });
        }
    });
}
