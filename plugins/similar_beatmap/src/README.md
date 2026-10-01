# 相似铺面插件


## 生命周期

用户输入 `beatmapID` -> 解析 `beatmapID` 获取 `.osu`文件  —> `.osu`文件查数据库获得多维难度数据 ->
`.osu`获得多维难度数据后，查询数据返回一个候选`beatmapID`数组 -> 返回拼装的text发送给。

