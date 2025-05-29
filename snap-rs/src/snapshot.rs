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
    pub timesteps: Vec<u32>, // x coords
    pub offsets: Vec<u32>,   // y coords, length same as `timesteps`
    pub size: u32,           // height (sweep distance)
    pub callstack: Vec<Frame>,
}

// Intermediate struct to help parse the structure of allocations.json
#[derive(Deserialize)]
pub struct RawAllocationData {
    pub timesteps: Vec<u32>,
    pub offsets: Vec<u32>,
    pub size: u32,
}

// Intermediate struct to help parse the structure of elements.json
// Each element in elements.json contains a list of frames.
#[derive(Deserialize)]
pub struct ElementData {
    pub frames: Vec<Frame>,
}
