作为一个 REPL (Read-Eval-Print Loop) 来观察显存 trace，需要提供以下功能，以便用户能够高效地探索和分析大量数据：

### 核心观察与过滤功能

1.  **加载与基本信息展示:**
    * **`load <path>`**: 加载显存 trace 数据文件 (`Vec<Allocation>`)。
    * **`info`**: 显示当前加载 trace 的基本统计信息，例如：总 Allocation 数量、内存总大小、时间范围（`min_timestep`, `max_timestep`）、最多的调用栈深度等。

2.  **筛选与过滤:**
    * **`filter_by_time <start_timestep> <end_timestep>`**: 筛选在指定时间范围内发生的 Allocation。
    * **`filter_by_size <min_size> <max_size>`**: 筛选指定大小范围内的 Allocation。
    * **`filter_by_function <function_name_regex>`**: 筛选调用栈中包含特定函数名的 Allocation。支持正则表达式进行模糊匹配。
    * **`filter_by_filename <filename_regex>`**: 筛选调用栈中包含特定文件名的 Allocation。
    * **`filter_by_callstack_depth <min_depth> <max_depth>`**: 筛选特定调用栈深度的 Allocation。
    * **`reset_filters`**: 清除所有已应用的过滤器，返回到原始 trace 数据。

3.  **排序与查看:**
    * **`sort_by <field> [asc|desc]`**: 根据 `size`、`timesteps.first()` (分配时间)、`timesteps.last()` (回收时间)、`callstack.len()` (调用栈深度) 等字段进行排序。
    * **`head <n>` / `tail <n>`**: 查看当前过滤/排序结果的前 `n` 条或后 `n` 条 Allocation。
    * **`show <index>`**: 显示指定索引的单个 Allocation 的详细信息，包括 `timesteps`, `offsets`, `size`, 和完整的 `callstack`。

### 统计与聚合功能

1.  **时间线分析:**
    * **`peak_memory_at_time <timestep>`**: 显示在特定时间点上的显存峰值（包括所有未释放的 Allocation）。
    * **`memory_usage_over_time <interval_size>`**: 按时间间隔（例如，每100个 timestep）统计总显存使用量，可以用于生成趋势图的数据。

2.  **热点分析:**
    * **`top_allocators_by_size <n>`**: 显示前 `n` 个分配总大小最大的函数或文件（按调用栈最顶层或整个调用栈聚合）。
    * **`top_allocators_by_count <n>`**: 显示前 `n` 个分配次数最多的函数或文件。
    * **`memory_leak_candidates <threshold_time>`**: 识别长时间未释放的 Allocation（例如，分配时间早于 `threshold_time` 且尚未释放的）。

3.  **调用栈分析:**
    * **`common_callstacks <n>`**: 显示最常见的 `n` 个调用栈及其对应的总分配大小和次数。
    * **`callstack_tree [depth]`**: 生成一个可交互的调用栈树状结构，显示每个函数的累积显存消耗。用户可以展开/折叠节点。

### 诊断与调试功能

1.  **`diff <timestep1> <timestep2>`**: 比较两个时间点之间显存的变化，例如，哪些 Allocation 在这段时间内被创建，哪些被释放，哪些仍然存在。
2.  **`find_unfreed_since <timestep>`**: 查找从特定时间点之后分配但尚未被释放的 Allocation。
3.  **`diagnose_spike <timestep>`**: 在给定时间点如果出现显存峰值，尝试找出导致峰值的主要 Allocation。

### 导出与可视化辅助

1.  **`export_filtered <format> <path>`**: 将当前过滤/排序的结果导出到文件，例如 CSV、JSON。
2.  **`export_summary <format> <path>`**: 导出统计聚合结果（如热点分析）。
3.  **`generate_plot_data <type> <path>`**: 生成用于外部绘图工具的数据，例如：
    * `generate_plot_data timeline_memory <path>`: 生成显存随时间变化的折线图数据。
    * `generate_plot_data sunburst_callstack <path>`: 生成用于 Sunburst 图（饼图）的调用栈数据。

### REPL 交互与辅助功能

1.  **`help [command]`**: 显示命令帮助信息。
2.  **`history`**: 显示历史命令。
3.  **`save_session <path>` / `load_session <path>`**: 保存和加载当前的 REPL 状态（例如，已加载的数据和应用的过滤器）。
4.  **`alias <name> <command_sequence>`**: 定义命令别名，方便执行常用操作。
5.  **`macro <name> <command_sequence>`**: 定义宏，执行一系列命令。
6.  **`quit` / `exit`**: 退出 REPL。

### 实现考虑

* **性能优化:** 面对数百万条数据，必须考虑索引、缓存和高效的数据结构，以确保过滤和聚合操作的响应速度。例如，可以预计算一些常用的聚合结果。
* **内存效率:** Rust 的所有权和借用机制天然有助于内存管理，但仍需注意避免不必要的复制。
* **用户体验:** 提供清晰的错误信息、进度指示，并考虑命令的自动补全功能。
* **可扩展性:** 设计一个易于添加新功能和分析方法的架构。

通过提供这些功能，用户可以灵活地在 REPL 中深入探索显存 trace 数据，定位内存泄漏、分析显存峰值原因、优化显存使用等。