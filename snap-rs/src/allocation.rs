use serde::Deserialize;

// Corresponds to the Python Frame dataclass
#[derive(Deserialize, Debug)]
pub struct Frame {
    pub name: String, // function name
    pub filename: String,
    pub line: u32,
}

// Corresponds to the Python Allocation dataclass
#[derive(Deserialize, Debug)]
pub struct Allocation {
    pub timesteps: Vec<u32>, // x coords, sorted
    pub offsets: Vec<u64>,   // y coords, length same as `timesteps`
    pub size: u64,           // height (sweep distance)
    pub callstack: Vec<Frame>,
    pub peak_mem: u64,
    pub peak_timestamps: Vec<u32>, // reaches its peak at these timestamps
}

impl Allocation {
    pub fn is_alive_in_interval(&self, start: u32, stop: u32) -> bool {
        self.is_alive_at(start) && self.is_alive_at(stop)
    }

    pub fn is_alive_at(&self, timestamp: u32) -> bool {
        self.timesteps[0] <= timestamp && timestamp <= *self.timesteps.last().unwrap()
    }

    pub fn start_end_time(&self) -> (u32, u32) {
        (self.timesteps[0], *self.timesteps.last().unwrap())
    }
}

// Intermediate struct to help parse the structure of allocations.json
#[derive(Deserialize)]
pub struct RawAllocationData {
    pub timesteps: Vec<u32>,
    pub offsets: Vec<u64>,
    pub size: u64,
}

// Intermediate struct to help parse the structure of elements.json
// Each element in elements.json contains a list of frames.
#[derive(Deserialize)]
pub struct ElementData {
    pub frames: Vec<Frame>,
}
