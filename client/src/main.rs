use std::{
    cell::UnsafeCell,
    rc::Rc,
    sync::{atomic::{AtomicUsize, AtomicU64}, Arc},
    time::Duration,
};

use config::{ClientConfig, COUNT_GRAIN_PRE_SEC, PACKET_SIZE};
use local_sync::semaphore::Semaphore;
use monoio::{
    io::{AsyncReadRentExt, AsyncWriteRentExt},
    net::TcpStream,
    RuntimeBuilder,
};

fn main() {
    let cfg = Arc::new(ClientConfig::parse());
    println!(
        r"Running ping pong client.
Packet size: {}
Connection count per core: {}; Global connection count: {}
QPS limit per core: {}; Global QPS limit: {}
Target: {}
CPU slot: {}",
        PACKET_SIZE,
        cfg.conns_per_core,
        cfg.conns_per_core * cfg.cores.len(),
        cfg.qps_per_core.unwrap_or(0),
        cfg.qps_per_core.unwrap_or(0) * cfg.cores.len(),
        cfg.target,
        config::format_cores(&cfg.cores)
    );
    assert!(
        cfg.qps_per_core.unwrap_or(COUNT_GRAIN_PRE_SEC as _) >= COUNT_GRAIN_PRE_SEC as _,
        "QPS limit should be more than COUNT_GRAIN_PRE_SEC"
    );

    // count will be shared across threads
    let count = Arc::new(AtomicUsize::new(0));
    let eps = Arc::new(AtomicU64::new(0));

    for cpu in cfg.cores.iter() {
        let cfg_ = cfg.clone();
        let cpu_ = *cpu as _;
        let count_ = count.clone();
        let eps_ = eps.clone();
        std::thread::spawn(move || {
            monoio::utils::bind_to_cpu_set(Some(cpu_)).unwrap();
            let mut rt = RuntimeBuilder::<monoio::IoUringDriver>::new().with_entries(2560).enable_timer().build().unwrap();
            rt.block_on(run_thread(count_, eps_, cfg_));
            println!("Thread {} finished", cpu_);
        });
    }

    // every second(not precise), we will print the status
    let mut count_last = 0;
    let instant = std::time::Instant::now();
    loop {
        std::thread::sleep(Duration::from_secs(1));
        let count_now = count.load(std::sync::atomic::Ordering::Relaxed);
        let eps_now = eps.load(std::sync::atomic::Ordering::Relaxed);
        let eps_sec = instant.elapsed().as_secs_f32();
        println!(
            "{:.3}: NAdd: {}; NSum: {}; NAverage: {:.3}, LatencyAverage: {:.3} us",
            eps_sec,
            count_now - count_last,
            count_now,
            count_now as f32 / eps_sec,
            eps_now as f32 / count_now as f32,
        );
        count_last = count_now;
    }
}

// start new tasks for each connection on the same thread
async fn run_thread(count: Arc<AtomicUsize>, eps: Arc<AtomicU64>, cfg: Arc<ClientConfig>) {
    let mut hdrs = Vec::with_capacity(cfg.conns_per_core);

    // count_tls and sem will be shared across tasks.
    let count_tls = Rc::new(UnsafeCell::new(0));
    let eps_tls = Rc::new(UnsafeCell::new(0));
    let grain_n = cfg.qps_per_core.unwrap_or(0) / COUNT_GRAIN_PRE_SEC as usize;
    let sem = cfg.qps_per_core.map(|_| Rc::new(Semaphore::new(grain_n)));

    for _ in 0..cfg.conns_per_core {
        hdrs.push(monoio::spawn(run_conn(
            count_tls.clone(),
            eps_tls.clone(),
            sem.clone(),
            cfg.target.clone(),
        )));
    }
    let mut interval = monoio::time::interval(Duration::from_secs(1) / COUNT_GRAIN_PRE_SEC);
    loop {
        interval.tick().await;
        let c = unsafe { &mut *count_tls.get() };
        let e = unsafe { &mut *eps_tls.get() };
        count.fetch_add(*c, std::sync::atomic::Ordering::Relaxed);
        *c = 0;
        eps.fetch_add(*e, std::sync::atomic::Ordering::Relaxed);
        *e = 0;
        if let Some(s) = sem.as_ref() {
            s.add_permits(grain_n);
        }
    }
}

async fn run_conn(
    count: Rc<UnsafeCell<usize>>,
    eps: Rc<UnsafeCell<u64>>,
    qps_per_conn: Option<Rc<Semaphore>>,
    target: String,
) {
    let mut buf = vec![0; PACKET_SIZE];
    let mut stream = TcpStream::connect(target).await.unwrap();

    loop {
        if let Some(s) = qps_per_conn.as_ref() {
            s.acquire().await.unwrap().forget();
        }

        let begin = std::time::Instant::now();
        let (w, buf_w) = stream.write_all(buf).await;
        if w.is_err() {
            // The connection is closed.
            println!("Write failed, connection exit");
            return;
        }
        let (r, buf_r) = stream.read_exact(buf_w).await;
        if r.is_err() {
            // The connection is closed.
            println!("Read failed, connection exit");
            return;
        }
        let eps_ = begin.elapsed().as_micros() as u64;
        buf = buf_r;
        unsafe {
            *count.get() += 1;
            *eps.get() += eps_;
        }
    }
}
