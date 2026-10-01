use kovi::{Bot, tokio};
use rosu_v2::Osu;

pub struct BotStatus {
    pub bot: Bot,
    pub rosu: Osu,
}

impl BotStatus {
    async fn new() -> Self {
        //rosu-v2 init
        let client_id: u64 = std::env::var("CLIENT_ID").unwrap().parse().unwrap();
        let client_secret = std::env::var("CLIENT_SECRET").unwrap();
        let osu = Osu::new(client_id, client_secret).await.unwrap();

        //bot init
        let driver_config = kovi_onebot::load_local_conf().unwrap();
        let driver = kovi_onebot::OneBotDriver::new(driver_config);

        let bot = kovi::build_bot!(driver; kovi_plugin_cmd,echo,issues,message_route);

        BotStatus {
            bot: bot,
            rosu: osu,
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
