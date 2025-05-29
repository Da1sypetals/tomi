import pickle


def process_alloc_data(device_trace, plot_segments=False, max_entries=15000):
    elements = []
    initially_allocated = []
    actions = []
    addr_to_alloc = {}

    # 第一阶段：处理事件流
    alloc_actions = {"alloc", "segment_alloc"} if not plot_segments else {"segment_alloc"}
    free_actions = {"free", "free_completed"} if not plot_segments else {"segment_free", "segment_free"}

    for idx, event in enumerate(device_trace):
        if event["action"] in alloc_actions:
            elements.append(event)
            addr_to_alloc[event["addr"]] = len(elements) - 1
            actions.append(len(elements) - 1)
        elif event["action"] in free_actions:
            if event["addr"] in addr_to_alloc:
                actions.append(addr_to_alloc[event["addr"]])
                del addr_to_alloc[event["addr"]]
            else:
                elements.append(event)
                initially_allocated.append(len(elements) - 1)
                actions.append(len(elements) - 1)

    # 第二阶段：构建初始内存状态（示例简化，需根据实际segment数据补充）
    # 这里假设没有预分配内存块，仅处理事件流中的初始状态
    # 实际需要根据segment/block数据补充类似JavaScript的初始分配逻辑

    # 第三阶段：模拟时间线
    current = []
    current_data = []
    data = []
    max_size = 0
    total_mem = 0
    total_summarized_mem = 0
    timestep = 0
    max_at_time = []
    summarized_mem = {"elem": "summarized", "timesteps": [], "offsets": [total_mem], "size": [], "color": 0}
    summarized_elems = set()

    def advance(n):
        nonlocal timestep
        summarized_mem["timesteps"].append(timestep)
        summarized_mem["offsets"].append(total_mem)
        summarized_mem["size"].append(total_summarized_mem)
        timestep += n
        for _ in range(n):
            max_at_time.append(total_mem + total_summarized_mem)

    # 处理初始分配
    for elem in reversed(initially_allocated):
        if elem < max_entries:
            # 添加到可视分配
            element = elements[elem]
            current.append(elem)
            color = elem
            data_entry = {
                "elem": elem,
                "timesteps": [timestep],
                "offsets": [total_mem],
                "size": element["size"],
                "color": color,
            }
            current_data.append(data_entry)
            data.append(data_entry)
            total_mem += element["size"]
        else:
            # 添加到汇总内存
            summarized_elems.add(elem)
            total_summarized_mem += elements[elem]["size"]

    # 处理动作序列
    for elem in actions:
        element = elements[elem]
        size = element["size"]

        if elem >= max_entries:
            # 处理汇总内存
            if elem in summarized_elems:
                advance(1)
                total_summarized_mem -= size
                summarized_elems.discard(elem)
            else:
                total_summarized_mem += size
                summarized_elems.add(elem)
                advance(1)
            continue

        # 查找当前分配
        try:
            idx = next(i for i in reversed(range(len(current))) if current[i] == elem)
        except StopIteration:
            # 新增分配
            current.append(elem)
            color = elem
            data_entry = {"elem": elem, "timesteps": [timestep], "offsets": [total_mem], "size": size, "color": color}
            current_data.append(data_entry)
            data.append(data_entry)
            total_mem += size
            advance(1)
        else:
            # 释放分配
            removed = current_data[idx]
            removed["timesteps"].append(timestep)
            removed["offsets"].append(removed["offsets"][-1])
            del current[idx]
            del current_data[idx]

            # 调整后续块位置
            if idx < len(current_data):
                for entry in current_data[idx:]:
                    entry["timesteps"].append(timestep)
                    entry["offsets"].append(entry["offsets"][-1])
                    entry["timesteps"].append(timestep + 3)
                    entry["offsets"].append(entry["offsets"][-1] - size)
                advance(3)

            total_mem -= size
            advance(1)

        max_size = max(max_size, total_mem + total_summarized_mem)

    # 收尾处理
    for entry in current_data:
        entry["timesteps"].append(timestep)
        entry["offsets"].append(entry["offsets"][-1])

    data.append(summarized_mem)

    return {
        "max_size": max_size,
        "allocations_over_time": data,
        "max_at_time": max_at_time,
        "summarized_mem": summarized_mem,
        "elements": elements,
    }


def get_trace(dump: dict):
    return dump["device_traces"]


path = "/home/da1sypetals/dev/torch-snapshot/snapshots/snapshot.pickle"
# unpickle

with open(path, "rb") as f:
    dump = pickle.load(f)

out = process_alloc_data(get_trace(dump)[0])


max_size = out["max_size"]
allocations_over_time = out["allocations_over_time"]
max_at_time = out["max_at_time"]
summarized_mem = out["summarized_mem"]
elements = out["elements"]


# save each to json:
# max_size: save to json
import json

with open("generated/max_size.json", "w") as f:
    f.write(json.dumps(max_size))

# allocations_over_time: save to json
with open("generated/allocations_over_time.json", "w") as f:
    f.write(json.dumps(allocations_over_time))

# max_at_time: save to json
with open("generated/max_at_time.json", "w") as f:
    f.write(json.dumps(max_at_time))

# summarized_mem: save to json
with open("generated/summarized_mem.json", "w") as f:
    f.write(json.dumps(summarized_mem))

# elements: save to json
with open("generated/elements.json", "w") as f:
    f.write(json.dumps(elements))


from icecream import ic

ic(len(elements))
ic(len(allocations_over_time))

"""
elements: 原始的 action 对象，保存了分配地址，callstack等信息，长度为n
allocation_over_time: 用来画图的东西。其中最后一项是summary，可以忽略，长度为n+1

"""
