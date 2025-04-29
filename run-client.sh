#!/bin/bash

server="127.0.0.1:40000"
timeout_duration=60.5
max_connections=100
step=10
name="benchmark"
cores=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --name)
      name="$2"
      shift 2
      ;;
    --cores)
      cores="$2"
      shift 2
      ;;
    --timeout)
      timeout_duration="$2"
      shift 2
      ;;
    --max-connections)
      max_connections="$2"
      shift 2
      ;;
    --step)
      step="$2"
      shift 2
      ;;
    --server)
      server="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 --name <name> --cores <cores> [--timeout <seconds>] [--max-connections <num>] [--step <num>] [--server <addr:port>]"
      exit 1
      ;;
  esac
done

if [ -z "$cores" ]; then
  echo "Error: --cores is required"
  echo "Usage: $0 --name <name> --cores <cores> [--timeout <seconds>] [--max-connections <num>] [--step <num>] [--server <addr:port>]"
  exit 1
fi

dir="data/${name}"
mkdir -p $dir
echo "conns,rps,lat" > $dir/all.csv

conns=$(seq $step $step $max_connections)
cmd="./target/release/client --target $server --cores $cores"

echo "Starting benchmark: $name"
echo "Server: $server"
echo "Cores: $cores"
echo "Max connections: $max_connections"
echo "Timeout: $timeout_duration seconds"
echo "Output directory: $dir"
echo ""

for i in $conns; do
    echo "Running with $i conns-per-core"
    output="$dir/$i.txt"
    timeout $timeout_duration $cmd --conns-per-core $i 2>&1 | tee $output

    rps=$(tail -n1 $output | awk '{print $7}' | tr -d ';')
    lat=$(tail -n1 $output | awk '{print $9}')
    echo "$i,$rps,$lat"
    echo ""
    echo "$i,$rps,$lat" >> $dir/all.csv
    sleep 5
done

echo "Benchmark complete. Results saved to $dir/all.csv"
