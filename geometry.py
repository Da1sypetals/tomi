from pydantic.dataclasses import dataclass
import json


@dataclass
class Allocation:
    timesteps: list[int]
    y: list[int]
    size: int


def load_allocations(path: str):
    with open(path) as f:
        data = json.load(f)
    return Allocation(data[])
