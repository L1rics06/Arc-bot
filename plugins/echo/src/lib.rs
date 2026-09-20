use kovi::PluginBuilder as plugin;
use kovi_onebot::*;

#[kovi::plugin]
async fn main() {
    plugin::on_msg(|event| async move {
        if event.borrow_text() == Some("hi") {
            event.reply("hi")
        }
    });
}
