# Tomi

A REPL tool to inspect PyTorch memory dump files. To generate a dump, see https://docs.pytorch.org/docs/stable/torch_cuda_memory.html

## Run
1. 生成一个snapshot: `/home/da1sypetals/dev/torch-snapshot/generate_snapshot/transformer.py`
2. 生成压缩的json dump: 
   ```sh
   # given PyTorch dump at ./snapshots/large/transformer.pickle
   python parse_dump.py -p snapshots/large/transformer.pickle -o ./dumpjson -d 0 -z
   # this outputs to ./snap.zip
   ```
3. 用snap-rs加载：
   ```sh
   cd snap-rs

   # then load the zipped dump
   cargo run -r --bin repl -- --zip ../dumpjson/snap.zip
   ```
4. 使用snap-rs
   ```
   tomi> help
   ```


---
# Notes for myself

1. 原本代码的瓶颈是每次拖动，放大缩小等都需要重新，然后render，render的逻辑是使用js写的，自然非常慢。
   1. 提升性能只需要把render的逻辑写成rust即可。
   2. 因此可以用某种python的GUI 库写一个viewer，然后在加载数据和reload的时候，调用rust extension写的重新加载函数。
2. 可以要求一个压缩的格式，然后在运行时在内存里解压，再反序列化到数据结构。

Equivalently run via:
```
RUST_LOG="off,snap_rs=info" cargo run --bin repl -- ...
```


# TODO
1. -v -b 输出bitmap， svg；指定resolution
2. i idx/inspect idx 检视特定index位置的alloc