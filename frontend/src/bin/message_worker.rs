use frontend::chat_bus::ChatBus;
use yew_agent::PublicWorker;

fn main() {
    ChatBus::register();
}
