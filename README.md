# My Fork Changes

- Added run_client.sh script to run the clients with multiple cores and connections and save the results to csv files.
- Added [tokio-uring](https://github.com/tokio-rs/tokio-uring) server.
- Added [compio](https://github.com/compio-rs/compio) server.
- Added [smol](https://github.com/smol-rs/smol) server.
- Added [may](https://github.com/Xudong-Huang/may) server.
- Added [nio](https://github.com/nio-rs/nio) server.
- Updated dependencies to latest versions.

To run the benchmark:

```bash
cargo build --release
# Run a server
./target/release/monoio-server --cores 1 2
# Run the client script
./run_client.sh
```

In the future, I'll also add more clients.


# Benchmark

TCP ping-pong(not echo) is a common benchmark for network applications.

We will use 1K ping-pong to test performance of different runtimes. Server will receive 1K data(and parse it in real applications) and reply 1K data.

## Max throughput
With given connections(enough for fully utilize CPU cores which latency is not important), we can measure the maximum throughput.

## Fixed QPS
We can measure the latency and CPU utilization of different runtimes.

## How to run
Run diffrent servers:
```
./target/release/monoio-server --cores 1
./target/release/glommio-server --cores 1
taskset -c 1 ./target/release/tokio-server --cores 1

./target/release/monoio-server --cores 1 2 3 4
./target/release/glommio-server --cores 1 2 3 4
taskset -c 1-4 ./target/release/tokio-server --cores 1 2 3 4
```

Run client:
```
./target/release/client --target 10.0.0.0:40000 --cores 1 2 3 4 --conns-per-core 150
```

## Benchmark result
Full result is available in [中文版本](https://github.com/bytedance/monoio/blob/master/docs/zh/benchmark.md) / [English version](https://github.com/bytedance/monoio/blob/master/docs/zh/benchmark.md).
