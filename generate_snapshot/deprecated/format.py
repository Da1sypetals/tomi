from typing import List, Literal, TypedDict


class Frame(TypedDict):
    filename: str
    line: int
    name: str


class Block(TypedDict):
    # A piece of memory returned from the allocator, or
    # current cached but inactive.
    size: int
    requested_size: int  # size requested during malloc, may be smaller than size due to rounding
    address: int
    state: Literal[
        "active_allocated",  # used by a tensor
        "active_awaiting_free",  # waiting for another stream to finish using this, then it will become free
        "inactive",
    ]  # free for reuse
    frames: List[Frame]  # stack trace from where the allocation occurred


class Segment(TypedDict):
    # Segments are memory returned from a cudaMalloc call.
    # The size of reserved memory is the sum of all Segments.
    # Segments are cached and reused for future allocations.
    # If the reuse is smaller than the segment, the segment
    # is split into more then one Block.
    # empty_cache() frees Segments that are entirely inactive.
    address: int
    total_size: int  #  cudaMalloc'd size of segment
    stream: int
    segment_type: Literal["small", "large"]  # 'large' (>1MB)
    allocated_size: int  # size of memory in use
    active_size: int  # size of memory in use or in active_awaiting_free state
    blocks: List[Block]


class TraceEntry(TypedDict):
    # When `torch.cuda.memory._record_memory_history()` is enabled,
    # the snapshot will contain TraceEntry objects that record each
    # action the allocator took.
    action: Literal[
        "alloc"  # memory allocated
        "free_requested",  # the allocated received a call to free memory
        "free_completed",  # the memory that was requested to be freed is now able to be used in future allocation calls
        "segment_alloc",  # the caching allocator ask cudaMalloc for more memory and added it as a segment in its cache
        "segment_free",  # the caching allocator called cudaFree to return memory to cuda possibly trying free up memory to allocate more segments or because empty_caches was called
        "oom",  # the allocator threw an OOM exception. 'size' is the requested number of bytes that did not succeed
        "snapshot",  # the allocator generated a memory snapshot useful to coorelate a previously taken snapshot with this trace
    ]
    addr: int  # not present for OOM
    frames: List[Frame]
    size: int
    stream: int
    device_free: int  # only present for OOM, the amount of memory cuda still reports to be free


class Snapshot(TypedDict):
    segments: List[Segment]
    device_traces: List[List[TraceEntry]]  # device_traces[device_id] is the trace for that device
