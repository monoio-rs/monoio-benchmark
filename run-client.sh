#!/bin/bash

# set -xe

tag=$1
cores=$2
dir=data/$tag-c$cores

mkdir -p $dir
echo "conns,rps,lat" >$dir/all.csv

server="127.0.0.1:40000"
conns=$(seq 10 10 100)
cores_start=8
cores_end=$(($cores_start + $cores - 1))
cores_list=$(seq $cores_start 1 $cores_end)
cmd="./target/release/client --target $server --cores $cores_list"

for i in $conns; do
    echo "Running with $i conns-per-core"
    output="$dir/$i.txt"
    timeout 60.5 $cmd --conns-per-core $i 2>&1 | tee $output

    rps=$(tail -n1 $output | awk '{print $7}')
    lat=$(tail -n1 $output | awk '{print $9}')
    echo "$i,$rps,$lat"
    echo ""
    echo "$i,$rps,$lat" >>$dir/all.csv
    sleep 10
done