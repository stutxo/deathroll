use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use yew_agent::{HandlerId, Public, Worker, WorkerLink};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    EventBusMsg(String),
}

pub struct FeedBus {
    link: WorkerLink<FeedBus>,
    subscribers: HashSet<HandlerId>,
}

impl Worker for FeedBus {
    type Message = ();
    type Input = Request;
    type Output = String;
    type Reach = Public<Self>;

    fn create(link: WorkerLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            Request::EventBusMsg(s) => {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, s.clone())
                }
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }

    fn name_of_resource() -> &'static str {
        "./assets/message_worker.js"
    }
}
