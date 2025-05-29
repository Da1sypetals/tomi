# REPL
--repl
--json --alloc allocations.json --elem elements.json

或者
--sqlite file.db

选择一个合适的schema，选择一个memory db, 支持SQL
- sqlbuild 建立sqlite表格
- sqlsave /path/to/file.db 保持sqlite文件
- sqlmode 进入sql模式
- sql .... 使用sql命令
（例如，字符串filter+sum大小，用sql实现）
需要一个把所有frames拼接在一起存储的字段，方便查找。
(好像没有适合的schema...)

frames可能需要直接拼接成varchar存储，而不是存储一个列表；用最长的作为varchar长度。
timesteps / offsets直接取最长的作为存储长度，然后用timesteps_len存储长度。

支持peak n，查找所有alloc中 max(offset)+size 第n大的，然后输出（这个时候alive的所有alloc的index）
需要建立每个timestamp哪些alloc活着 的索引


## 返回idx，提供-i返回完整alloc信息
按size递减排序topk   topk 10 : 在存活的alloc中，size最大的10个
按idx查看       i 1003 或者inspect 1003: 查看第1003个alloc的信息
xxxcommand > save.txt 保存输出      i 1003 > alloc_1003.txt : 查看第1003个alloc的信息并保存到alloc_1003.txt

### 在某个时间点
查看哪些alloc在这时间存活  @1331 查看1331时刻存活的alloc,只看index
                        @1331 -i 查看1331时刻存活的alloc,直接按offset从小到大列举。
在某个时间点按size递减排序topk    @1331 top 10 : 在时间1331，存活的alloc中，size最大的10个 （-i 列举）

<!-- time range filter, 提供起止时间，查看哪些alloc在这时间内完整存活 @1331-1320 查看1331-1320时刻完整存活的alloc信息 -->


help


## 其他工具
--plot-line
生成一个（显存随时间变化折线图），方便发现显存最高的时间，然后用REPL查看这个时间点。
