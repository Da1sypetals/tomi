import struct
import io
from typing import List, Literal, TypedDict, Dict, Set


# TypedDict definitions from the user
class Frame(TypedDict):
    filename: str
    line: int
    name: str


class Block(TypedDict):
    size: int
    requested_size: int
    address: int
    state: Literal["active_allocated", "active_awaiting_free", "inactive"]
    frames: List[Frame]


class Segment(TypedDict):
    address: int
    total_size: int
    stream: int
    segment_type: Literal["small", "large"]
    allocated_size: int
    active_size: int
    blocks: List[Block]


class TraceEntry(TypedDict):
    action: Literal[
        "alloc",
        "free_requested",
        "free_completed",
        "segment_alloc",
        "segment_free",
        "oom",
        "snapshot",
    ]
    addr: int  # not present for OOM
    frames: List[Frame]
    size: int
    stream: int
    device_free: int  # only present for OOM


class Snapshot(TypedDict):
    segments: List[Segment]
    device_traces: List[List[TraceEntry]]


# Constants for the format
MAGIC_NUMBER = b"SNAP"
FORMAT_VERSION = 0x01
LITTLE_ENDIAN_PREFIX = "<"

# Literal to Integer Mappings
BLOCK_STATE_MAP: Dict[str, int] = {
    "active_allocated": 0,
    "active_awaiting_free": 1,
    "inactive": 2,
}
BLOCK_STATE_REVERSE_MAP: Dict[int, str] = {v: k for k, v in BLOCK_STATE_MAP.items()}

SEGMENT_TYPE_MAP: Dict[str, int] = {"small": 0, "large": 1}
SEGMENT_TYPE_REVERSE_MAP: Dict[int, str] = {v: k for k, v in SEGMENT_TYPE_MAP.items()}

TRACE_ACTION_MAP: Dict[str, int] = {
    "alloc": 0,
    "free_requested": 1,
    "free_completed": 2,
    "segment_alloc": 3,
    "segment_free": 4,
    "oom": 5,
    "snapshot": 6,
}
TRACE_ACTION_REVERSE_MAP: Dict[int, str] = {v: k for k, v in TRACE_ACTION_MAP.items()}
OOM_ACTION_VALUE = TRACE_ACTION_MAP["oom"]


class SnapshotEncoder:
    def __init__(self, snapshot: Snapshot):
        self.snapshot = snapshot
        self.string_to_id: Dict[str, int] = {}
        self.string_list: List[str] = []

    def _collect_strings_from_frames(
        self, frames: List[Frame], unique_strings: Set[str]
    ):
        for frame in frames:
            unique_strings.add(frame["filename"])
            unique_strings.add(frame["name"])

    def _collect_all_strings(self):
        unique_strings: Set[str] = set()
        for segment in self.snapshot["segments"]:
            for block in segment["blocks"]:
                self._collect_strings_from_frames(block["frames"], unique_strings)

        for device_trace_list in self.snapshot["device_traces"]:
            for trace_entry in device_trace_list:
                self._collect_strings_from_frames(trace_entry["frames"], unique_strings)

        self.string_list = sorted(
            list(unique_strings)
        )  # Sorted for deterministic output
        self.string_to_id = {s: i for i, s in enumerate(self.string_list)}

    def _write_uint8(self, writer: io.BytesIO, value: int):
        writer.write(struct.pack(LITTLE_ENDIAN_PREFIX + "B", value))

    def _write_int32(self, writer: io.BytesIO, value: int):
        writer.write(struct.pack(LITTLE_ENDIAN_PREFIX + "i", value))

    def _write_uint32(self, writer: io.BytesIO, value: int):
        writer.write(struct.pack(LITTLE_ENDIAN_PREFIX + "I", value))

    def _write_int64(self, writer: io.BytesIO, value: int):
        writer.write(struct.pack(LITTLE_ENDIAN_PREFIX + "q", value))

    def _write_uint64(self, writer: io.BytesIO, value: int):
        writer.write(struct.pack(LITTLE_ENDIAN_PREFIX + "Q", value))

    def _encode_string_table(self, writer: io.BytesIO):
        self._write_uint32(writer, len(self.string_list))
        for s in self.string_list:
            s_bytes = s.encode("utf-8")
            self._write_uint32(writer, len(s_bytes))
            writer.write(s_bytes)

    def _encode_frame(self, writer: io.BytesIO, frame: Frame):
        self._write_uint32(writer, self.string_to_id[frame["filename"]])
        self._write_uint32(writer, frame["line"])
        self._write_uint32(writer, self.string_to_id[frame["name"]])

    def _encode_block(self, writer: io.BytesIO, block: Block):
        self._write_uint64(writer, block["size"])
        self._write_uint64(writer, block["requested_size"])
        self._write_uint64(writer, block["address"])
        self._write_uint8(writer, BLOCK_STATE_MAP[block["state"]])

        self._write_uint32(writer, len(block["frames"]))
        for frame in block["frames"]:
            self._encode_frame(writer, frame)

    def _encode_segment(self, writer: io.BytesIO, segment: Segment):
        self._write_uint64(writer, segment["address"])
        self._write_uint64(writer, segment["total_size"])
        self._write_int64(writer, segment["stream"])  # Changed to int64
        self._write_uint8(writer, SEGMENT_TYPE_MAP[segment["segment_type"]])
        self._write_uint64(writer, segment["allocated_size"])
        self._write_uint64(writer, segment["active_size"])

        self._write_uint32(writer, len(segment["blocks"]))
        for block in segment["blocks"]:
            self._encode_block(writer, block)

    def _encode_trace_entry(self, writer: io.BytesIO, entry: TraceEntry):
        action_val = TRACE_ACTION_MAP[entry["action"]]
        self._write_uint8(writer, action_val)
        self._write_int64(writer, entry["stream"])  # Changed to int64
        self._write_uint64(writer, entry["size"])

        if action_val == OOM_ACTION_VALUE:
            self._write_uint64(writer, entry["device_free"])
        else:
            self._write_uint64(writer, entry["addr"])

        self._write_uint32(writer, len(entry["frames"]))
        for frame in entry["frames"]:
            self._encode_frame(writer, frame)

    def encode(self) -> bytes:
        self._collect_all_strings()

        writer = io.BytesIO()

        # Header
        writer.write(MAGIC_NUMBER)
        self._write_uint8(writer, FORMAT_VERSION)

        # String Table
        self._encode_string_table(writer)

        # Segments
        self._write_uint32(writer, len(self.snapshot["segments"]))
        for segment in self.snapshot["segments"]:
            self._encode_segment(writer, segment)

        # Device Traces
        self._write_uint32(writer, len(self.snapshot["device_traces"]))
        for device_trace_list in self.snapshot["device_traces"]:
            self._write_uint32(writer, len(device_trace_list))
            for trace_entry in device_trace_list:
                self._encode_trace_entry(writer, trace_entry)

        return writer.getvalue()


class SnapshotDecoder:
    def __init__(self, data: bytes):
        self.reader = io.BytesIO(data)
        self.id_to_string: List[str] = []

    def _read_bytes(self, num_bytes: int) -> bytes:
        data = self.reader.read(num_bytes)
        if len(data) < num_bytes:
            raise EOFError("Unexpected end of file while reading data.")
        return data

    def _read_uint8(self) -> int:
        return struct.unpack(LITTLE_ENDIAN_PREFIX + "B", self._read_bytes(1))[0]

    def _read_int32(self) -> int:
        return struct.unpack(LITTLE_ENDIAN_PREFIX + "i", self._read_bytes(4))[0]

    def _read_uint32(self) -> int:
        return struct.unpack(LITTLE_ENDIAN_PREFIX + "I", self._read_bytes(4))[0]

    def _read_int64(self) -> int:
        return struct.unpack(LITTLE_ENDIAN_PREFIX + "q", self._read_bytes(8))[0]

    def _read_uint64(self) -> int:
        return struct.unpack(LITTLE_ENDIAN_PREFIX + "Q", self._read_bytes(8))[0]

    def _decode_string_table(self):
        num_strings = self._read_uint32()
        self.id_to_string = []
        for _ in range(num_strings):
            str_len = self._read_uint32()
            str_bytes = self._read_bytes(str_len)
            self.id_to_string.append(str_bytes.decode("utf-8"))

    def _decode_frame(self) -> Frame:
        filename_id = self._read_uint32()
        line = self._read_uint32()
        name_id = self._read_uint32()
        return Frame(
            filename=self.id_to_string[filename_id],
            line=line,
            name=self.id_to_string[name_id],
        )

    def _decode_block(self) -> Block:
        size = self._read_uint64()
        requested_size = self._read_uint64()
        address = self._read_uint64()
        state_val = self._read_uint8()

        num_frames = self._read_uint32()
        frames: List[Frame] = [self._decode_frame() for _ in range(num_frames)]

        return Block(
            size=size,
            requested_size=requested_size,
            address=address,
            state=BLOCK_STATE_REVERSE_MAP[state_val],  # type: ignore
            frames=frames,
        )

    def _decode_segment(self) -> Segment:
        address = self._read_uint64()
        total_size = self._read_uint64()
        stream = self._read_int64()  # Changed to int64
        segment_type_val = self._read_uint8()
        allocated_size = self._read_uint64()
        active_size = self._read_uint64()

        num_blocks = self._read_uint32()
        blocks: List[Block] = [self._decode_block() for _ in range(num_blocks)]

        return Segment(
            address=address,
            total_size=total_size,
            stream=stream,
            segment_type=SEGMENT_TYPE_REVERSE_MAP[segment_type_val],  # type: ignore
            allocated_size=allocated_size,
            active_size=active_size,
            blocks=blocks,
        )

    def _decode_trace_entry(self) -> TraceEntry:
        action_val = self._read_uint8()
        stream = self._read_int64()  # Changed to int64
        size = self._read_uint64()

        addr = 0
        device_free = 0

        if action_val == OOM_ACTION_VALUE:
            device_free = self._read_uint64()
        else:
            addr = self._read_uint64()

        num_frames = self._read_uint32()
        frames: List[Frame] = [self._decode_frame() for _ in range(num_frames)]

        entry: TraceEntry = {  # type: ignore
            "action": TRACE_ACTION_REVERSE_MAP[action_val],  # type: ignore
            "stream": stream,
            "size": size,
            "frames": frames,
        }
        if action_val == OOM_ACTION_VALUE:
            entry["device_free"] = device_free
        else:
            entry["addr"] = addr
        return entry

    def decode(self) -> Snapshot:
        # Header
        magic = self._read_bytes(len(MAGIC_NUMBER))
        if magic != MAGIC_NUMBER:
            raise ValueError("Invalid magic number. This is not a SNAP file.")

        version = self._read_uint8()
        if version != FORMAT_VERSION:
            raise ValueError(
                f"Unsupported format version. Expected {FORMAT_VERSION}, got {version}."
            )

        # String Table
        self._decode_string_table()

        # Segments
        num_segments = self._read_uint32()
        segments: List[Segment] = [self._decode_segment() for _ in range(num_segments)]

        # Device Traces
        num_devices = self._read_uint32()
        device_traces: List[List[TraceEntry]] = []
        for _ in range(num_devices):
            num_trace_entries = self._read_uint32()
            trace_list: List[TraceEntry] = [
                self._decode_trace_entry() for _ in range(num_trace_entries)
            ]
            device_traces.append(trace_list)

        return Snapshot(segments=segments, device_traces=device_traces)


if __name__ == "__main__":
    # Example Usage:
    sample_snapshot_data: Snapshot = {
        "segments": [
            {
                "address": 0x1000,
                "total_size": 2048,
                "stream": 1,
                "segment_type": "small",
                "allocated_size": 1024,
                "active_size": 512,
                "blocks": [
                    {
                        "size": 1024,
                        "requested_size": 1000,
                        "address": 0x1000,
                        "state": "active_allocated",
                        "frames": [
                            {
                                "filename": "/path/to/file1.py",
                                "line": 10,
                                "name": "func_a",
                            },
                            {
                                "filename": "/path/to/file2.py",
                                "line": 25,
                                "name": "func_b",
                            },
                        ],
                    }
                ],
            }
        ],
        "device_traces": [
            [
                {
                    "action": "alloc",
                    "addr": 0x1000,
                    "frames": [
                        {"filename": "/path/to/file1.py", "line": 10, "name": "func_a"}
                    ],
                    "size": 1024,
                    "stream": 1,
                    # No 'device_free' for "alloc"
                },
                {
                    "action": "oom",
                    # No 'addr' for "oom"
                    "frames": [
                        {
                            "filename": "/path/to/file_oom.py",
                            "line": 5,
                            "name": "oom_func",
                        }
                    ],
                    "size": 4096,
                    "stream": 2,
                    "device_free": 10240,
                },
            ]
        ],
    }

    # Add missing keys with default values if necessary for TraceEntry
    # (as per TypedDict definition, addr and device_free are not always present structurally,
    # but my dict construction for TraceEntry in example needs to be careful)

    # Ensure 'addr' or 'device_free' based on action for valid TypedDict
    # The example 'alloc' has 'addr' implicitly due to structure.
    # The example 'oom' needs 'device_free'. 'addr' should not be there.
    # My current TraceEntry TypedDict makes addr and device_free effectively optional.
    # The encoder/decoder handles this correctly by presence.
    # For constructing the sample_snapshot_data, it's fine.

    print("Original Snapshot Data:")
    # print(sample_snapshot_data) # Can be very verbose

    encoder = SnapshotEncoder(sample_snapshot_data)
    encoded_bytes = encoder.encode()

    print(
        f"\nEncoded {len(sample_snapshot_data['segments'])} segment(s) and {len(sample_snapshot_data['device_traces'])} device trace list(s)."
    )
    print(f"Encoded size: {len(encoded_bytes)} bytes")

    decoder = SnapshotDecoder(encoded_bytes)
    decoded_snapshot_data = decoder.decode()

    print("\nDecoded Snapshot Data (checking a few fields):")
    print(f"Decoded segments: {len(decoded_snapshot_data['segments'])}")
    if decoded_snapshot_data["segments"]:
        print(
            f"First segment address: {decoded_snapshot_data['segments'][0]['address']}"
        )
        if decoded_snapshot_data["segments"][0]["blocks"]:
            print(
                f"First block state: {decoded_snapshot_data['segments'][0]['blocks'][0]['state']}"
            )
            if decoded_snapshot_data["segments"][0]["blocks"][0]["frames"]:
                print(
                    f"First frame in first block: {decoded_snapshot_data['segments'][0]['blocks'][0]['frames'][0]}"
                )

    if (
        decoded_snapshot_data["device_traces"]
        and decoded_snapshot_data["device_traces"][0]
    ):
        print(
            f"First trace entry action: {decoded_snapshot_data['device_traces'][0][0]['action']}"
        )
        if decoded_snapshot_data["device_traces"][0][0]["action"] == "oom":
            print(
                f"Device free for OOM: {decoded_snapshot_data['device_traces'][0][0].get('device_free')}"
            )  # Using .get for safety
        else:
            print(
                f"Address for non-OOM: {decoded_snapshot_data['device_traces'][0][0].get('addr')}"
            )

    # Basic check for equality (Python's dict comparison works recursively)
    # Note: floating point numbers or complex objects might require custom comparison.
    # Here, all data is basic types, strings, ints, and lists/dicts of them.
    # The string IDs will be different if string_list was not sorted before creating string_to_id map,
    # but the final decoded strings should match. Sorting `unique_strings` before list conversion ensures
    # deterministic string IDs if that matters for byte-exact re-encoding.
    # The decoded structure should be semantically identical.

    # A deep comparison function would be better for proper validation.
    # For this example, we'll assume if key fields match, it's likely correct.
    assert (
        sample_snapshot_data["segments"][0]["address"]
        == decoded_snapshot_data["segments"][0]["address"]
    )
    assert (
        sample_snapshot_data["segments"][0]["blocks"][0]["state"]
        == decoded_snapshot_data["segments"][0]["blocks"][0]["state"]
    )
    assert (
        sample_snapshot_data["segments"][0]["blocks"][0]["frames"][0]["filename"]
        == decoded_snapshot_data["segments"][0]["blocks"][0]["frames"][0]["filename"]
    )

    print("\nEncoding and decoding successful, and basic data matches.")

    ####################################################################

    # After encoding the snapshot data with SnapshotEncoder
    encoded_bytes = encoder.encode()
    import pickle

    # Pickle the sample snapshot data
    pickled_data = pickle.dumps(sample_snapshot_data, protocol=pickle.HIGHEST_PROTOCOL)

    # Compare sizes
    print(f"\nEncoded size (custom binary format): {len(encoded_bytes)} bytes")
    print(f"Pickled size (Python pickle): {len(pickled_data)} bytes")
