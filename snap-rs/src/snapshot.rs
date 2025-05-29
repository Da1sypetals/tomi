use serde::Deserialize;

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
    pub(crate) offsets: Vec<u32>,   // y coords, length same as `timesteps`
    pub(crate) size: u32,           // height (sweep distance)
    pub(crate) callstack: Vec<Frame>,
}

// Intermediate struct to help parse the structure of allocations.json
#[derive(Deserialize)]
pub struct RawAllocationData {
    pub(crate) timesteps: Vec<u32>,
    pub(crate) offsets: Vec<u32>,
    pub(crate) size: u32,
}

// Intermediate struct to help parse the structure of elements.json
// Each element in elements.json contains a list of frames.
#[derive(Deserialize)]
pub struct ElementData {
    pub(crate) frames: Vec<Frame>,
}
