//! 编号的格式化与解析。
//!
//! 编号以 `#003` 形式展示与落盘（也是文件名）；解析时兼容 `3` / `#3` / `#003`。

/// 编号上限：三位数，够用且便于展示。
pub const MAX_ID: u32 = 999;

/// 编号渲染为 `#003`。
pub fn fmt_id(id: u32) -> String {
    format!("#{id:03}")
}

/// 从存储文件名（即 `Issue::id`）解析出编号，兼容带 / 不带 `#` 两种写法。
pub fn parse_stored_id(name: &str) -> Result<u32, std::num::ParseIntError> {
    name.trim_start_matches('#').parse::<u32>()
}

/// 用户输入的编号引用（`3` / `#3` / `#003`）→ 编号。
pub fn parse_user_id(input: &str) -> Result<u32, crate::Error> {
    let digits = input.trim().trim_start_matches('#');
    let id: u32 = digits
        .parse()
        .map_err(|_| crate::Error::Invalid(crate::local_text::hint::bad_id(input)))?;
    if id == 0 {
        return Err(crate::Error::Invalid(
            crate::local_text::hint::ZERO_ID.to_owned(),
        ));
    }
    Ok(id)
}
