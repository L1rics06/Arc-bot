//! arc-bot 入口：只做装配，不含业务逻辑。
//!
//! 启动顺序：
//! 1. 加载 `.env`；
//! 2. 读取 kovi 配置（OneBot 连接参数），构建驱动与 Bot；
//! 3. 挂载插件（命令路由 + kovi 自带的管理命令插件）；
//! 4. `bot.run()` —— 阻塞运行，直到进程被终止。
//!
//! 业务侧的初始化（共享 osu! 客户端、命令表、数据目录）都发生在
//! [`cmd_router`] 插件启动时，见 `cmd_router/src/lib.rs`。

use kovi::Bot;

#[kovi::tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let driver_config = match kovi_onebot::load_local_conf() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("读取 kovi.conf.toml 失败：{e:#}");
            eprintln!("请确认工作目录正确，或删除该文件让 kovi 重新引导生成。");
            std::process::exit(1);
        }
    };
    let driver = kovi_onebot::OneBotDriver::new(driver_config);

    let bot: Bot = kovi::build_bot!(driver; kovi_plugin_cmd, cmd_router);
    bot.run().await;
}
