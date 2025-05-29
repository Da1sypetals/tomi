use std::convert::TryFrom;
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path; // For u8 to enum conversion if needed, though direct match is fine.

// --- Constants ---
const MAGIC_NUMBER_BYTES: &[u8; 4] = b"SNAP";
const EXPECTED_FORMAT_VERSION: u8 = 0x01;
const OOM_ACTION_VALUE_U8: u8 = 5; // From Python: TRACE_ACTION_MAP["oom"]

// --- Rust Data Structures (derived from Python TypedDicts and encoder logic) ---

#[derive(Debug, Clone)]
pub struct Frame {
    pub filename: String,
    pub line: u32, // Encoded as uint32
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockState {
    ActiveAllocated,    // 0
    ActiveAwaitingFree, // 1
    Inactive,           // 2
}

impl BlockState {
    fn from_u8(value: u8) -> Result<Self, DecodeError> {
        match value {
            0 => Ok(BlockState::ActiveAllocated),
            1 => Ok(BlockState::ActiveAwaitingFree),
            2 => Ok(BlockState::Inactive),
            _ => Err(DecodeError::InvalidBlockStateValue(value)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub size: u64,           // Encoded as uint64
    pub requested_size: u64, // Encoded as uint64
    pub address: u64,        // Encoded as uint64
    pub state: BlockState,
    pub frames: Vec<Frame>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SegmentType {
    Small, // 0
    Large, // 1
}

impl SegmentType {
    fn from_u8(value: u8) -> Result<Self, DecodeError> {
        match value {
            0 => Ok(SegmentType::Small),
            1 => Ok(SegmentType::Large),
            _ => Err(DecodeError::InvalidSegmentTypeValue(value)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub address: u64,    // Encoded as uint64
    pub total_size: u64, // Encoded as uint64
    pub stream: i64,     // Encoded as int64
    pub segment_type: SegmentType,
    pub allocated_size: u64, // Encoded as uint64
    pub active_size: u64,    // Encoded as uint64
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub enum TraceAction {
    Alloc { addr: u64 },         // 0
    FreeRequested { addr: u64 }, // 1
    FreeCompleted { addr: u64 }, // 2
    SegmentAlloc { addr: u64 },  // 3
    SegmentFree { addr: u64 },   // 4
    Oom { device_free: u64 },    // 5, Encoded as uint64
    Snapshot { addr: u64 },      // 6
}

#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub action: TraceAction,
    pub frames: Vec<Frame>,
    pub size: u64,   // Encoded as uint64
    pub stream: i64, // Encoded as int64
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub segments: Vec<Segment>,
    pub device_traces: Vec<Vec<TraceEntry>>,
}

// --- Custom Error Type for Decoding ---
#[derive(Debug)]
pub enum DecodeError {
    Io(io::Error),
    InvalidMagicNumber,
    UnsupportedFormatVersion(u8),
    Utf8(std::string::FromUtf8Error),
    InvalidStringId(u32, usize), // id, table_len
    InvalidBlockStateValue(u8),
    InvalidSegmentTypeValue(u8),
    InvalidTraceActionValue(u8),
    UnexpectedEof,
}

impl From<io::Error> for DecodeError {
    fn from(err: io::Error) -> Self {
        if err.kind() == io::ErrorKind::UnexpectedEof {
            DecodeError::UnexpectedEof
        } else {
            DecodeError::Io(err)
        }
    }
}

impl From<std::string::FromUtf8Error> for DecodeError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        DecodeError::Utf8(err)
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Io(e) => write!(f, "IO error: {}", e),
            DecodeError::InvalidMagicNumber => write!(f, "Invalid magic number. Not a SNAP file."),
            DecodeError::UnsupportedFormatVersion(v) => write!(
                f,
                "Unsupported format version. Expected {}, got {}.",
                EXPECTED_FORMAT_VERSION, v
            ),
            DecodeError::Utf8(e) => write!(f, "UTF-8 decoding error: {}", e),
            DecodeError::InvalidStringId(id, len) => {
                write!(f, "Invalid string ID: {} (table len: {})", id, len)
            }
            DecodeError::InvalidBlockStateValue(v) => write!(f, "Invalid block state value: {}", v),
            DecodeError::InvalidSegmentTypeValue(v) => {
                write!(f, "Invalid segment type value: {}", v)
            }
            DecodeError::InvalidTraceActionValue(v) => {
                write!(f, "Invalid trace action value: {}", v)
            }
            DecodeError::UnexpectedEof => write!(f, "Unexpected end of file while reading data."),
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DecodeError::Io(e) => Some(e),
            DecodeError::Utf8(e) => Some(e),
            _ => None,
        }
    }
}

// --- Reader Helper ---
struct SnapshotReader<R: Read> {
    reader: R,
    string_table: Vec<String>,
}

impl<R: Read> SnapshotReader<R> {
    fn new(reader: R) -> Self {
        SnapshotReader {
            reader,
            string_table: Vec::new(),
        }
    }

    fn read_exact_bytes(&mut self, count: usize) -> Result<Vec<u8>, DecodeError> {
        let mut buf = vec![0; count];
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_u8(&mut self) -> Result<u8, DecodeError> {
        let mut buf = [0u8; 1];
        self.reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_u32_le(&mut self) -> Result<u32, DecodeError> {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_u64_le(&mut self) -> Result<u64, DecodeError> {
        let mut buf = [0u8; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    fn read_i64_le(&mut self) -> Result<i64, DecodeError> {
        let mut buf = [0u8; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(i64::from_le_bytes(buf))
    }

    fn decode_header(&mut self) -> Result<(), DecodeError> {
        let magic = self.read_exact_bytes(MAGIC_NUMBER_BYTES.len())?;
        if magic != MAGIC_NUMBER_BYTES {
            return Err(DecodeError::InvalidMagicNumber);
        }
        let version = self.read_u8()?;
        if version != EXPECTED_FORMAT_VERSION {
            return Err(DecodeError::UnsupportedFormatVersion(version));
        }
        Ok(())
    }

    fn decode_string_table_and_store(&mut self) -> Result<(), DecodeError> {
        let num_strings = self.read_u32_le()?;
        self.string_table = Vec::with_capacity(num_strings as usize);
        for _ in 0..num_strings {
            let str_len = self.read_u32_le()?;
            let str_bytes = self.read_exact_bytes(str_len as usize)?;
            self.string_table.push(String::from_utf8(str_bytes)?);
        }
        Ok(())
    }

    fn get_string(&self, id: u32) -> Result<&String, DecodeError> {
        self.string_table
            .get(id as usize)
            .ok_or_else(|| DecodeError::InvalidStringId(id, self.string_table.len()))
    }

    fn decode_frame(&mut self) -> Result<Frame, DecodeError> {
        let filename_id = self.read_u32_le()?;
        let line = self.read_u32_le()?;
        let name_id = self.read_u32_le()?;

        Ok(Frame {
            filename: self.get_string(filename_id)?.clone(),
            line,
            name: self.get_string(name_id)?.clone(),
        })
    }

    fn decode_frames(&mut self) -> Result<Vec<Frame>, DecodeError> {
        let num_frames = self.read_u32_le()?;
        let mut frames = Vec::with_capacity(num_frames as usize);
        for _ in 0..num_frames {
            frames.push(self.decode_frame()?);
        }
        Ok(frames)
    }

    fn decode_block(&mut self) -> Result<Block, DecodeError> {
        let size = self.read_u64_le()?;
        let requested_size = self.read_u64_le()?;
        let address = self.read_u64_le()?;
        let state_val = self.read_u8()?;
        let state = BlockState::from_u8(state_val)?;
        let frames = self.decode_frames()?;

        Ok(Block {
            size,
            requested_size,
            address,
            state,
            frames,
        })
    }

    fn decode_blocks(&mut self) -> Result<Vec<Block>, DecodeError> {
        let num_blocks = self.read_u32_le()?;
        let mut blocks = Vec::with_capacity(num_blocks as usize);
        for _ in 0..num_blocks {
            blocks.push(self.decode_block()?);
        }
        Ok(blocks)
    }

    fn decode_segment(&mut self) -> Result<Segment, DecodeError> {
        let address = self.read_u64_le()?;
        let total_size = self.read_u64_le()?;
        let stream = self.read_i64_le()?;
        let segment_type_val = self.read_u8()?;
        let segment_type = SegmentType::from_u8(segment_type_val)?;
        let allocated_size = self.read_u64_le()?;
        let active_size = self.read_u64_le()?;
        let blocks = self.decode_blocks()?;

        Ok(Segment {
            address,
            total_size,
            stream,
            segment_type,
            allocated_size,
            active_size,
            blocks,
        })
    }

    fn decode_segments(&mut self) -> Result<Vec<Segment>, DecodeError> {
        let num_segments = self.read_u32_le()?;
        let mut segments = Vec::with_capacity(num_segments as usize);
        for _ in 0..num_segments {
            segments.push(self.decode_segment()?);
        }
        Ok(segments)
    }

    fn decode_trace_entry(&mut self) -> Result<TraceEntry, DecodeError> {
        let action_val = self.read_u8()?;
        let stream = self.read_i64_le()?;
        let size = self.read_u64_le()?;

        let action = match action_val {
            0 => TraceAction::Alloc {
                addr: self.read_u64_le()?,
            },
            1 => TraceAction::FreeRequested {
                addr: self.read_u64_le()?,
            },
            2 => TraceAction::FreeCompleted {
                addr: self.read_u64_le()?,
            },
            3 => TraceAction::SegmentAlloc {
                addr: self.read_u64_le()?,
            },
            4 => TraceAction::SegmentFree {
                addr: self.read_u64_le()?,
            },
            OOM_ACTION_VALUE_U8 => TraceAction::Oom {
                device_free: self.read_u64_le()?,
            },
            6 => TraceAction::Snapshot {
                addr: self.read_u64_le()?,
            },
            _ => return Err(DecodeError::InvalidTraceActionValue(action_val)),
        };
        let frames = self.decode_frames()?;
        Ok(TraceEntry {
            action,
            frames,
            size,
            stream,
        })
    }

    fn decode_trace_entries(&mut self) -> Result<Vec<TraceEntry>, DecodeError> {
        let num_entries = self.read_u32_le()?;
        let mut entries = Vec::with_capacity(num_entries as usize);
        for _ in 0..num_entries {
            entries.push(self.decode_trace_entry()?);
        }
        Ok(entries)
    }

    fn decode_device_traces(&mut self) -> Result<Vec<Vec<TraceEntry>>, DecodeError> {
        let num_devices = self.read_u32_le()?;
        let mut device_traces = Vec::with_capacity(num_devices as usize);
        for _ in 0..num_devices {
            device_traces.push(self.decode_trace_entries()?);
        }
        Ok(device_traces)
    }
}

// --- Public Decode Function ---
pub fn decode_snapshot_from_file(file_path: impl AsRef<Path>) -> Result<Snapshot, DecodeError> {
    let file = File::open(file_path)?;
    let buf_reader = BufReader::new(file);
    let mut snapshot_reader = SnapshotReader::new(buf_reader);

    // 1. Decode Header
    snapshot_reader.decode_header()?;

    // 2. Decode String Table (and store it in the reader)
    snapshot_reader.decode_string_table_and_store()?;

    // 3. Decode Segments
    let segments = snapshot_reader.decode_segments()?;

    // 4. Decode Device Traces
    let device_traces = snapshot_reader.decode_device_traces()?;

    Ok(Snapshot {
        segments,
        device_traces,
    })
}

#[test]
fn test_basic() {
    let python_encoder_output_path = "../snapshots/memory_snapshot_example_encoded.snap";
    match decode_snapshot_from_file(python_encoder_output_path) {
        Ok(snapshot) => {
            println!("Successfully decoded snapshot!");
            println!("========== Stats ==========");
            println!("Number of segments: {}", snapshot.segments.len());
            if !snapshot.segments.is_empty() {
                println!("First segment address: {:#X}", snapshot.segments[0].address);
                if !snapshot.segments[0].blocks.is_empty() {
                    println!(
                        "First block state: {:?}",
                        snapshot.segments[0].blocks[0].state
                    );
                }
            }

            println!();
            println!("Number of device traces: {}", snapshot.device_traces.len());
            if !snapshot.device_traces.is_empty() && !snapshot.device_traces[0].is_empty() {
                println!(
                    "First trace entry action: {:?}",
                    snapshot.device_traces[0][0].action
                );
            }
            println!("========== Stats End ==========");
            // For more detailed output, you can iterate through the structures
            // println!("{:#?}", snapshot); // Full debug print
        }
        Err(e) => {
            eprintln!("Failed to decode snapshot: {}", e);
        }
    }
}
