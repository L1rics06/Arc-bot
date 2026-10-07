use kovi::{Bot, tokio};
use rosu_v2::Osu;

pub struct BotStatus {
    pub bot: Bot,
    /// 凭据没配好时为 None，用到的插件需要自己判断
    pub rosu: Option<Osu>,
}

// todo：增加一个数据共享类的插件 解决bot状态插件间读取的问题
impl BotStatus {
    async fn new() -> Self {
        //rosu-v2 init，失败只降级不中断启动
        let rosu = match serde_beatmap::osu_from_env().await {
            Ok(osu) => Some(osu),
            Err(e) => {
                eprintln!("rosu 初始化失败，依赖 osu! API 的命令将不可用：{e:#}");
                None
            }
        };

        //bot init
        let driver_config = kovi_onebot::load_local_conf().unwrap();
        let driver = kovi_onebot::OneBotDriver::new(driver_config);

        let bot = kovi::build_bot!(driver; kovi_plugin_cmd,echo,issues,message_route);

        BotStatus {
            bot: bot,
            rosu: rosu,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let bot_status = BotStatus::new().await;
    bot_status.bot.run().await;
    Ok(())
}
