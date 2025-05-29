选择一个合适的schema，选择一个memory db, 支持SQL
- sqlmode进入sql模式
- sql .... 使用sql命令
需要一个把所有frames拼接在一起存储的字段，方便查找。
(好像没有适合的schema...)


支持peak n，查找所有alloc中 max(offset)+size 第n大的，然后输出（这个时候alive的所有alloc的index）
需要建立每个timestamp哪些alloc活着 的索引