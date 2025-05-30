1. 原本代码的瓶颈是每次拖动，放大缩小等都需要重新，然后render，render的逻辑是使用js写的，自然非常慢。
   1. 提升性能只需要把render的逻辑写成rust即可。
   2. 因此可以用某种python的GUI 库写一个viewer，然后在加载数据和reload的时候，调用rust extension写的重新加载函数。
2. 可以要求一个压缩的格式，然后在运行时在内存里解压，再反序列化到数据结构。

Equivalently run via:
```
RUST_LOG="off,snap_rs=info" cargo run --bin repl -- ...
```
