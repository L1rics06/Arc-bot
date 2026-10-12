//! beatmap_api 面向用户的文案集中处。
//!
//! 约定：所有会出现在 QQ 消息里的固定字符串都写在这里，其余模块只引用常量。

/// 命令用法。
pub mod usage {
    /// `/map` 的用法。
    pub const MAP: &str = "/map <beatmapID>";
}

/// 各类提示。
pub mod hint {
    /// 缺少 beatmapID。
    pub fn need_id() -> String {
        format!("缺少 beatmapID，用法：{}", super::usage::MAP)
    }
    /// beatmapID 不是数字。
    pub fn bad_id(raw: &str) -> String {
        format!("beatmapID `{raw}` 不是合法的数字")
    }
    /// beatmapID 为 0。
    pub const ZERO_ID: &str = "beatmapID 需为非零正整数";
    /// API 找不到该谱面。
    pub fn not_found(map_id: u32) -> String {
        format!("找不到 beatmapID `{map_id}`，请确认编号是否正确")
    }
    /// 谱面缺少 mapset 信息（被删 / graveyard）。
    pub fn missing_mapset(map_id: u32) -> String {
        format!("谱面 {map_id} 没有可用的谱面集信息（可能已被删除）")
    }
    /// API 未配置。
    pub fn api_not_configured() -> String {
        "osu! API 未初始化，/map 暂不可用（检查 .env 的 CLIENT_ID / CLIENT_SECRET）".to_owned()
    }
}

/// 谱面信息渲染用到的字段名与分隔符。
pub mod render {
    /// 谱面集区块标题。
    pub const SET_HEADER: &str = "== 谱面集信息 ==";
    /// 当前难度区块标题。
    pub const DIFF_HEADER: &str = "=== 当前难度 ===";
    /// 字段名（冒号后跟值）。
    pub const TITLE: &str = "标题";
    pub const CREATOR: &str = "作者";
    pub const STATUS: &str = "状态";
    pub const VERSION: &str = "难度名";
    pub const STARS: &str = "星数";
    pub const BPM: &str = "BPM";
    pub const DIFFICULTY: &str = "CS/AR/OD/HP";
}

/// `.osu` 文件下载相关的日志与提示。
pub mod osu_file {
    /// 下载失败的通用前缀。
    pub fn download_failed(detail: &str) -> String {
        format!("获取 .osu 文件失败：{detail}")
    }
    /// 请求失败。
    pub fn request_failed(url: &str, detail: &str) -> String {
        format!("请求 {url} 失败：{detail}")
    }
    /// HTTP 状态码非 2xx。
    pub fn bad_status(url: &str, status: &str) -> String {
        format!("下载 {url} 返回 {status}（谱面不存在或已被删除）")
    }
    /// 读取响应体失败。
    pub fn read_body_failed(url: &str, detail: &str) -> String {
        format!("读取 {url} 响应失败：{detail}")
    }
    /// 写盘失败。
    pub fn write_failed(path: &str, detail: &str) -> String {
        format!("写入 {path} 失败：{detail}")
    }
    /// 读取本地文件失败。
    pub fn read_failed(path: &str, detail: &str) -> String {
        format!("读取 {path} 失败：{detail}")
    }
    /// Difficulty 段字段非法。
    pub fn bad_field(key: &str, value: &str) -> String {
        format!("Difficulty 段字段 `{key}` 的值 `{value}` 无法解析")
    }
}

/// 缓存相关。
pub mod cache {
    /// 读写失败。
    pub fn failed(detail: &str) -> String {
        format!("缓存读写失败：{detail}")
    }
}

/// 凭据与 API 调用相关。
pub mod client {
    /// 缺少环境变量。
    pub fn missing_env(name: &str) -> String {
        format!("未设置环境变量 {name}")
    }
    /// CLIENT_ID 不是数字。
    pub fn bad_client_id(detail: &str) -> String {
        format!("CLIENT_ID 不是合法的数字：{detail}")
    }
    /// 认证失败。
    pub fn auth_failed(detail: &str) -> String {
        format!("认证失败，请检查 CLIENT_ID / CLIENT_SECRET：{detail}")
    }
    /// 查询 API 失败。
    pub fn api_failed(detail: &str) -> String {
        format!("查询 osu! API 失败：{detail}")
    }
}
