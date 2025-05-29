```rust
// Corresponds to the Python Frame dataclass

#[derive(Deserialize, Debug)]

pub struct Frame {

    pub(crate) name: String, // function name

    pub(crate) filename: String,

    pub(crate) line: u32,

}



// Corresponds to the Python Allocation dataclass

#[derive(Deserialize, Debug)]

pub struct Allocation {

    pub(crate) timesteps: Vec<u32>, // x coords

    pub(crate) offsets: Vec<u32>,   // y coords

    pub(crate) size: u32,           // height (sweep distance)

    pub(crate) callstack: Vec<Frame>,

}
```





我们用Vec<Allocation>来表示一个程序的显存消耗的trace。其中timesteps的第一个数字是被分配的时间，最后一个数字是被回收的时间，timestep的长度不定。对每个Allocation，offset和timestep一样长，是对应的timestep时候除了自己之外已被分配的显存(bytes), size是这个分配的大小。由于Allocation可能有几百万个，我们无法可视化（太卡了）。你觉得如果制作一个repl让用户来观察这个显存trace，需要提供哪些功能？