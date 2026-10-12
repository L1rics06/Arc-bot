//! `/map` 命令：注册与渲染。

mod handler;
pub mod render;

pub use handler::register;
pub use render::render;
