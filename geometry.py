from pydantic.dataclasses import dataclass
import json


@dataclass()
class Frame:
    name: str  # function name
    filename: str
    line: int


@dataclass
class Allocation:
    timesteps: list[int]
    offsets: list[int]
    size: int
    callstack: list[Frame]


def load_allocations(alloc_path: str, elements_path: str):
    with open(alloc_path) as f:
        allocations = json.load(f)

    with open(elements_path) as f:
        elements = json.load(f)

    callstacks = [[Frame(**frame) for frame in elem["frames"]] for elem in elements]

    return [
        Allocation(
            timesteps=alloc["timesteps"],
            offsets=alloc["offsets"],
            size=alloc["size"],
            callstack=callstack,
        )
        for alloc, callstack in zip(allocations, callstacks)
    ]


if __name__ == "__main__":
    alloc_path = "snapshots/allocations.json"
    elements_path = "snapshots/elements.json"
    allocations = load_allocations(alloc_path, elements_path)
    print(allocations)
