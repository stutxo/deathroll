use frontend::services::feed_bus::FeedBus;
use yew_agent::PublicWorker;

fn main() {
    FeedBus::register();
}
