//! 文件型 JSON 存储抽象。
//!
//! 约定：
//! - 每个业务模块一个目录，每个实体一个 `.json` 文件；
//! - 写入使用"临时文件 + rename"的原子方式，避免进程中断产生半个文件；
//! - 读取时非法 JSON 直接报错（带文件路径），不静默吞掉。

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::{Error, Result};

/// JSON 文件存储。
#[derive(Debug, Clone)]
pub struct JsonStore {
    dir: PathBuf,
}

impl JsonStore {
    /// 绑定目录（不创建，写入时自动创建父目录）。
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// 存储目录。
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// 校验实体名：只允许文件名字符，杜绝 `../` 之类的路径穿越。
    ///
    /// 名字最终会变成文件名，因此不允许路径分隔符、上跳、空名与控制字符。
    fn safe_name<'n>(&self, name: &'n str) -> Result<&'n str> {
        let ok = !name.is_empty()
            && name != "."
            && name != ".."
            && !name.contains(['/', '\\', '\0'])
            && !name.chars().any(char::is_control);
        if ok {
            Ok(name)
        } else {
            Err(Error::Storage(format!("非法的存储名 `{name}`")))
        }
    }

    /// 实体对应的文件路径（会校验名字）。
    pub fn path_of(&self, name: &str) -> Result<PathBuf> {
        Ok(self.dir.join(format!("{}.json", self.safe_name(name)?)))
    }

    /// 读取一个实体；文件不存在返回 `None`。
    pub async fn get<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>> {
        let path = self.path_of(name)?;
        if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|e| Error::Storage(format!("读取 {} 失败：{e}", path.display())))?;
        let value = serde_json::from_slice(&bytes)
            .map_err(|e| Error::Storage(format!("解析 {} 失败：{e}", path.display())))?;
        Ok(Some(value))
    }

    /// 读取全部实体（按文件名排序；只处理 `.json`）。
    ///
    /// 单个文件损坏会带上文件名报错，避免"数据坏了却装作没事"。
    pub async fn list<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut names = self.list_names().await?;
        names.sort();

        let mut out = Vec::with_capacity(names.len());
        for name in names {
            if let Some(v) = self.get(&name).await? {
                out.push(v);
            }
        }
        Ok(out)
    }

    /// 列出全部实体文件名（不含扩展名），按文件系统顺序返回。
    pub async fn list_names(&self) -> Result<Vec<String>> {
        let mut dir = match tokio::fs::read_dir(&self.dir).await {
            Ok(dir) => dir,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(Error::Storage(format!(
                    "读取目录 {} 失败：{e}",
                    self.dir.display()
                )));
            }
        };

        let mut names = Vec::new();
        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|e| Error::Storage(format!("遍历目录 {} 失败：{e}", self.dir.display())))?
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                names.push(stem.to_owned());
            }
        }
        Ok(names)
    }

    /// 写入实体（原子替换，自动创建目录）。
    pub async fn save<T: Serialize>(&self, name: &str, value: &T) -> Result<()> {
        let path = self.path_of(name)?;

        tokio::fs::create_dir_all(&self.dir)
            .await
            .map_err(|e| Error::Storage(format!("创建目录 {} 失败：{e}", self.dir.display())))?;

        let json = serde_json::to_vec_pretty(value)
            .map_err(|e| Error::Storage(format!("序列化 {name} 失败：{e}")))?;

        let tmp = self.dir.join(format!(".{name}.json.tmp"));
        tokio::fs::write(&tmp, &json)
            .await
            .map_err(|e| Error::Storage(format!("写入 {} 失败：{e}", tmp.display())))?;
        tokio::fs::rename(&tmp, &path)
            .await
            .map_err(|e| Error::Storage(format!("替换 {} 失败：{e}", path.display())))?;
        Ok(())
    }

    /// 删除实体；文件不存在时返回 `Ok(false)`。
    pub async fn remove(&self, name: &str) -> Result<bool> {
        let path = self.path_of(name)?;
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(Error::Storage(format!("删除 {} 失败：{e}", path.display()))),
        }
    }

    /// 确保目录存在。
    pub async fn ensure_dir(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.dir)
            .await
            .map_err(|e| Error::Storage(format!("创建目录 {} 失败：{e}", self.dir.display())))
    }
}
