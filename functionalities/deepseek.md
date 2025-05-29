为了高效地观察和分析包含数百万个 `Allocation` 的显存 Trace，REPL 应提供以下核心功能，重点关注查询、聚合和性能优化：

---

### **1. 时间线概览（Overview）**
- **功能**：展示全局统计信息，避免直接渲染海量数据。
- **输出内容**：
  - 总时间范围：`[min_timestep, max_timestep]`
  - 峰值显存占用及时间点：`peak_memory = X bytes at t=Y`
  - 分配总量/释放总量：`total_allocations = N, total_deallocations = M`
  - 显存生命周期分布直方图（按时间分桶）

---

### **2. 时间点查询（Time Slice Query）**
- **命令**：`query_time t`
- **功能**：查询特定时间点 `t` 的显存状态。
- **输出内容**：
  - 总显存占用：`current_memory = sum(active_allocations.size)`
  - 活跃分配数量：`active_allocations = K`
  - 按大小排序的活跃分配列表（分页展示）：
    ```plaintext
    [ID: 123, size: 10 MB, offset: 0xABCD, callstack: funcA@file:line]
    [ID: 456, size: 5 MB, offset: 0xEF01, callstack: funcB@file:line]
    ...
    ```

---

### **3. 分配过滤（Filter Allocations）**
- **命令**：
  - 按大小：`filter size_min=1MB size_max=10MB`
  - 按生命周期：`filter lifetime_min=1000`
  - 按调用栈：`filter function=conv2d`（匹配函数名或文件名）
- **功能**：过滤符合条件的分配，返回聚合结果。
- **输出内容**：
  - 匹配的分配数量及总大小：`matches: 25000 (total 1.2 GB)`
  - 可选分页查看详情（如 `show details page=1`）

---

### **4. 分配详情（Inspect Allocation）**
- **命令**：`inspect id=12345`
- **功能**：查看单个分配的详细信息。
- **输出内容**：
  ```plaintext
  Allocation ID: 12345
  Size: 50 MB
  Lifetime: [start=1000, end=5500]
  Callstack:
    [0] forward@model.py:120
    [1] conv2d@layers.py:45
    [2] ... 
  ```

---

### **5. 碎片分析（Fragmentation Report）**
- **命令**：`fragmentation at_time t=3000`
- **功能**：分析指定时间点的显存碎片。
- **输出内容**：
  - 总碎片率：`fragmentation = 35%`
  - 空闲区域列表（按大小排序）：
    ```plaintext
    [0x0000 - 0x0FFF]: 4 KB
    [0x2000 - 0x2FFF]: 4 KB
    ...
    ```
  - 最大连续空闲块：`largest_free_block = 12 MB`

---

### **6. 调用栈聚合（Callstack Aggregation）**
- **命令**：`aggregate_callstack depth=3`
- **功能**：按调用栈前缀聚合分配（如顶层3帧）。
- **输出内容**：
  ```plaintext
  | Rank | Calls | Total Size | Callstack Prefix        |
  | ---- | ----- | ---------- | ----------------------- |
  | 1    | 12000 | 1.5 GB     | train > step > forward  |
  | 2    | 8000  | 0.8 GB     | eval > forward > conv2d |
  ...
  ```

---

### **7. 峰值分析（Peak Memory Analysis）**
- **命令**：`analyze_peak`
- **功能**：自动定位峰值时间点，并分析其组成。
- **输出内容**：
  ```plaintext
  Peak at t=7500: 9.8 GB
  Top allocations by size:
    [ID: 88888] 2.0 GB (forward@model.py:120)
    [ID: 77777] 1.5 GB (conv2d@layers.py:45)
  ...
  ```

---

### **8. 时间区间扫描（Time Range Scan）**
- **命令**：`scan_range t_start=5000 t_end=6000`
- **功能**：分析区间内的显存变化趋势。
- **输出内容**：
  - 区间内显存占用曲线（ASCII 折线图）
  - 分配/释放事件统计：`allocs=350, deallocs=280`
  - 净显存变化：`net_change = +700 MB`

---

### **性能优化设计**
1. **预计算索引**：
   - 构建全局时间线事件索引（`(timestep, allocation_id, event_type)`）。
   - 为调用栈建立倒排索引（函数名 → 分配ID列表）。
   - 按分配大小排序的索引（用于快速范围查询）。

2. **惰性加载**：
   - 查询时只加载元数据，详情（如完整调用栈）按需展开。

3. **分页输出**：
   - 任何可能返回大量结果的命令默认返回摘要，需显式请求详情（如 `show full`）。

4. **近似统计**：
   - 对海量数据使用采样或分桶统计（如碎片率计算）。

---

### **示例 REPL 会话**
```plaintext
> overview
[OVERVIEW] Time: [0, 15000] | Allocs: 2.1M | Peak: 12.3 GB at t=7500

> analyze_peak
[PEAK] t=7500 (12.3 GB)
Top 5 allocations:
  [ID: 88888] 2.0 GB (forward@model.py:120)
  [ID: 77777] 1.5 GB (conv2d@layers.py:45)
  ...

> query_time t=7500
[QUERY] t=7500: 12.3 GB used by 24,580 allocations
Show top 10? [Y/n] Y
  [ID: 88888] 2.0 GB @ offset=0x1a000 (conv2d@layers.py:45)
  ...

> filter size_min=100MB function=conv2d
[FILTER] 1,240 matches (total 4.8 GB)
Type "show details" to list.

> inspect id=88888
[INSPECT] ID: 88888
Size: 2.0 GB | Lifetime: [7450, 7600]
Callstack:
  [0] forward@model.py:120
  [1] conv2d@layers.py:45
  ...
```

---

通过以上设计，用户可快速定位关键问题（如峰值、碎片、特定函数的内存消耗），同时避免因渲染原始数据导致的性能瓶颈。