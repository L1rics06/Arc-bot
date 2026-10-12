//! 内置命令注册入口。

pub mod help;

use crate::registry::Registry;

/// 把全部内置命令登记进注册表。
///
/// `/help` 必须读最终命令表，因此由调用方在业务命令登记完之后调用。
pub fn register_all<C: Send + Sync + 'static>(registry: &mut Registry<C>) {
    help::register(registry);
}
