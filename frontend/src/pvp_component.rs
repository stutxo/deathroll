use crate::Route;

use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};

use std::vec;
use web_sys::{window, Text};
use web_sys::{Element, MouseEvent};
use yew::platform::pinned::mpsc::{self, UnboundedReceiver, UnboundedSender};
use yew::platform::spawn_local;
use yew_router::prelude::*;

use yew::{html, AttrValue, Callback, Component, Html, NodeRef};

pub enum Msg {
    Roll,
}

pub struct PvPComponent {
    roll_amount: String,
    player_turn: bool,
    game_over: bool,
    game_start: bool,
    node_ref: NodeRef,
    feed: Vec<String>,
    tx: UnboundedSender<Message>,
}

impl PvPComponent {
    fn scroll_top(&self) {
        let node_ref = self.node_ref.clone();

        spawn_local(async move {
            let chat_main = node_ref.cast::<Element>().unwrap();

            chat_main.set_scroll_top(chat_main.scroll_height());
        })
    }
    // fn add_to_feed(&self, slash_roll: String) -> String {
    //     let prev_turn = {
    //         self.display_roll
    //             .len()
    //             .checked_sub(2)
    //             .map(|i| self.display_roll[i])
    //             .unwrap()
    //     };

    //     let end = self.add_end();

    //     let borrowed_roll = self.roll_amount.to_string();
    //     let space = " (1-";
    //     let prev = prev_turn.to_string();

    //     let is_rolling = slash_roll.clone() + &borrowed_roll + space + &prev + &end;

    //     is_rolling
    // }
    // fn add_end(&self) -> String {
    //     if self.game_over == true && self.computer_result == true {
    //         let end = ") \u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}";
    //         end.to_string()
    //     } else if self.game_over == true && self.computer_result == false {
    //         let end = ") \u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}";
    //         end.to_string()
    //     } else {
    //         let end = ") \u{1F3B2}";
    //         end.to_string()
    //     }
    // }
}

impl Component for PvPComponent {
    type Message = Msg;
    type Properties = ();
    fn create(_ctx: &yew::Context<Self>) -> Self {
        let window = window().unwrap();
        let location = window.location();
        let url = location.href().unwrap();

        let url_split: Vec<&str> = url.split('/').collect();

        let start = "ws://".to_owned();
        let host = url_split[2];
        let ws = "/ws/";
        let game_id = url_split[3];
        let roll_amount = url_split[4];

        let full_url = start + host + ws + game_id;

        log::debug!("{:?}", game_id);

        let (game_tx, mut game_rx): (UnboundedSender<Message>, UnboundedReceiver<Message>) =
            mpsc::unbounded();

        let ws = WebSocket::open(&full_url).unwrap();
        let (mut write, mut read) = ws.split();

        spawn_local(async move {
            while let Some(message) = game_rx.next().await {
                log::debug!("{:?}", message);
                write.send(message).await.unwrap();
            }
        });

        spawn_local(async move {
            while let Some(result) = read.next().await {
                let document = web_sys::window().unwrap().document().unwrap();
                let chat_box = document.get_element_by_id("log").unwrap();

                let msg = format!("{:?}", result.unwrap());

                let start_index = msg.find('"').unwrap();
                let end_index = msg[start_index + 1..].find('"').unwrap();
                let number_string = &msg[start_index + 1..end_index + start_index + 1];

                chat_box.set_text_content(Some(&number_string));
                log::debug!("{:?}", number_string);
            }
        });

        game_tx
            .send_now(Message::Text(String::from(roll_amount)))
            .unwrap();

        Self {
            roll_amount: roll_amount.to_string(),
            player_turn: true,
            game_over: false,
            game_start: true,
            node_ref: NodeRef::default(),
            feed: Vec::new(),
            tx: game_tx,
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let navigator = ctx.link().navigator().unwrap();
        let home = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));

        let roll_emoji = '\u{1F3B2}';
        //let replay = '\u{1F504}';
        let skull = '\u{1F480}';

        let on_click = ctx.link().callback(move |_: MouseEvent| Msg::Roll);

        let window = window().unwrap();
        let location = window.location();
        let url = location.href().unwrap();
       

        html! {
                        <div class="app-body">
                        <header class="header">
                        <div>
                        <button onclick={home} class="title-button">{"deathroll.gg "}{skull}{roll_emoji}</button>
                        <h1>{"1v1 me bruv"}</h1>
                        {"To invite someone to play, give this URL: "}{url}
                        </div>
                        <br/>
                       </header>
                       <div class="msger">
                       <main class="msger-chat" id="chat-main" ref={self.node_ref.clone()}>
                       <div class="dets">

                       
                       <div id="log"></div>
                      
                    //   {
                    //       self.feed.clone().into_iter().map(|name| {
                    //       html!{

                    //       <div class="msg" key={name.clone()}>
                    //        {" "}{name}
                    //        </div>

                    //       }
                    //       }).collect::
                    //       <Html>
                    //          ()
                    //          }
                             </div>

                       </main>
                       </div>


                       <div>
                       if self.game_over == false{<button hidden=true>{""}</button>
                        } else {
                    //    <button onclick={reset_game} class="replay-button">{replay}</button>
                        }

                       </div>
                       <div>


                    <button onclick={on_click} class="roll-button">{roll_emoji}</button>



            </div>
                    </div>
                    }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Roll => {
                let roll = "rolling".to_string();
                self.tx.send_now(Message::Text(String::from(roll))).unwrap();

                self.game_start = false;
                self.player_turn = false;
                self.scroll_top();

                let slash_roll: String = "[player]: /roll ".to_owned();
                let space = " 1-";
                let value = self.roll_amount.to_string();

                let is_rolling = slash_roll.clone() + space + &value;
                self.feed.push(is_rolling);

                true
            }
        }
    }
}
