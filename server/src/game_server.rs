use rand::Rng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc;
use uuid::Uuid;

pub type PlayerId = Uuid;
pub type GameId = String;
pub type Msg = String;

#[derive(Debug)]
pub enum Command {
    Connect {
        player_tx: mpsc::UnboundedSender<Msg>,
        game_id: GameId,
        player_id: PlayerId,
    },

    Disconnect {
        player_id: PlayerId,
        game_id: GameId,
    },

    Message {
        msg: Msg,
        player_id: PlayerId,
        game_id: GameId,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct GameMsg {
    roll_msg: Vec<String>,
}

#[derive(Debug)]
pub struct GameState {
    roll: u32,
    player_count: Arc<AtomicUsize>,
    player_1: Uuid,
    player_2: Option<Uuid>,
    player_turn: String,
    game_start: bool,
    start_roll: u32,
    game_over: bool,
    game_msg: GameMsg,
}

#[derive(Debug)]
pub struct GameServer {
    sessions: HashMap<PlayerId, mpsc::UnboundedSender<Msg>>,
    game_id: HashMap<GameId, HashSet<PlayerId>>,
    cmd_rx: mpsc::UnboundedReceiver<Command>,
    game_state: HashMap<GameId, GameState>,
}
impl GameServer {
    pub fn new() -> (Self, GameServerHandle) {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        (
            Self {
                sessions: HashMap::new(),
                game_id: HashMap::new(),
                cmd_rx,
                game_state: HashMap::new(),
            },
            GameServerHandle { cmd_tx },
        )
    }

    pub fn update_game_feed(&self, game_id: &str) {
        let game = self.game_state.get(game_id).unwrap();
        let msg = serde_json::to_string(&game.game_msg).unwrap();

        if let Some(game_id) = self.game_id.get(game_id) {
            for player_ids in game_id {
                if let Some(cmd_tx) = self.sessions.get(player_ids) {
                    let _ = cmd_tx.send(msg.clone());
                }
            }
        }
    }

    pub fn send_status_message(&self, player_id: PlayerId, msg: impl Into<String>) {
        let msg = msg.into();

        if let Some(cmd_tx) = self.sessions.get(&player_id) {
            let _ = cmd_tx.send(msg.clone());
        }
    }

    pub async fn new_turn(&mut self, player_id: PlayerId, roll: String, game_id: GameId) {
        let p1 = "\u{1F9D9}\u{200D}\u{2642}\u{FE0F}";
        let p2 = "\u{1F9DF}";

        match self.game_state.get(&game_id) {
            Some(_) => {
                if let Some(game_state) = self
                    .game_state
                    .iter()
                    .find_map(|(game, game_state)| game.contains(&game_id).then_some(game_state))
                {
                    self.update_game_feed(&game_id);
                    if game_state.player_turn == player_id.to_string()
                        && !game_state.game_over
                        && game_state.game_start
                    {
                        let re = Regex::new(r"\d").unwrap();

                        let contains_number = re.is_match(&roll);

                        if !contains_number {
                            let roll_between = game_state.roll.clone();
                            let roll = roll_die(game_state.roll).await;
                            if roll != 1 {
                                //handle player 1 turn
                                if player_id == game_state.player_1 {
                                    let msg = format!("{p1} {roll} \u{1F3B2} (1-{roll_between})");
                                    self.game_state.entry(game_id.clone()).and_modify(
                                        |game_state| {
                                            game_state.roll = roll;
                                            game_state.game_msg.roll_msg.push(msg);

                                            if let Some(player_2) = game_state.player_2.clone() {
                                                game_state.player_turn = player_2.to_string()
                                            }
                                        },
                                    );
                                    //handle player 2 turn
                                } else if Some(player_id) == game_state.player_2 {
                                    let msg = format!("{p2} {roll} \u{1F3B2} (1-{roll_between})");
                                    self.game_state.entry(game_id.clone()).and_modify(
                                        |game_state| {
                                            game_state.roll = roll;
                                            game_state.game_msg.roll_msg.push(msg);
                                            game_state.player_turn = game_state.player_1.to_string()
                                        },
                                    );
                                }
                            } else {
                                let defeat = format!("\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480} DEFEAT!!!");
                                let victory = format!("\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6} VICTORY!!!");
                                //handle player 1 death
                                if player_id == game_state.player_1 {
                                    //send victory status message to player 2
                                    let player2 = Some(game_state.player_2).unwrap();
                                    self.send_status_message(player2.unwrap(), victory);
                                    //send defeat status message to player 1
                                    let player1 = game_state.player_1;

                                    self.send_status_message(player1, defeat);
                                    //deathroll feed update
                                    let msg = format!(
                                        "{p1} 1 \u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480} \u{1F3B2} (1-{roll_between})");
                                    self.game_state.entry(game_id.clone()).and_modify(
                                        |game_state| {
                                            game_state.roll = roll;
                                            game_state.game_msg.roll_msg.push(msg);
                                            game_state
                                                .game_msg
                                                .roll_msg
                                                .push("\u{1F3C1} gameover!!".to_string());
                                            game_state.game_over = true;
                                        },
                                    );
                                    //handle player 1 death
                                } else if Some(player_id) == game_state.player_2 {
                                    //send victory message to player 1
                                    let player1 = game_state.player_1;
                                    self.send_status_message(player1, victory);
                                    //send defeat status message to player 2
                                    let player2 = Some(game_state.player_2).unwrap();
                                    self.send_status_message(player2.unwrap(), defeat);
                                    //deathroll feed update
                                    let msg = format!(
                                        "{p2} 1 \u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480} \u{1F3B2} (1-{roll_between})");
                                    self.game_state.entry(game_id.clone()).and_modify(
                                        |game_state| {
                                            game_state.roll = roll;
                                            game_state.game_msg.roll_msg.push(msg);
                                            game_state
                                                .game_msg
                                                .roll_msg
                                                .push("\u{1F3C1} gameover!!".to_string());
                                            game_state.game_over = true;
                                        },
                                    );
                                }
                            }
                            self.update_game_feed(&game_id);
                        }
                    } else if game_state.game_start == false && game_state.roll != 1 {
                    //do nothing
                    } else if game_state.game_over {
                        self.update_game_feed(&game_id);
                    } else if game_state.game_start == true
                        && game_state.roll == game_state.start_roll
                    {
                        if Some(player_id) == game_state.player_2 {
                            let msg = format!("waiting for {p1} to start the game..");
                            self.send_status_message(player_id, msg);
                        }

                        let player_id = game_state.player_1;

                        let msg = format!("roll to start the game..",);
                        self.send_status_message(player_id, msg);
                    } else {
                        let msg = format!("It's not your turn...");
                        self.send_status_message(player_id, msg);
                    }
                }
            }

            None => {
                let start_roll: u32 = match roll.trim().parse::<u32>() {
                    Ok(parsed_input) => parsed_input,

                    Err(_) => 1,
                };

                let game_msg = GameMsg {
                    roll_msg: Vec::new(),
                };
                if start_roll != 1 {
                    let game_state = GameState {
                        roll: start_roll,
                        player_count: Arc::new(AtomicUsize::new(1)),
                        player_1: player_id,
                        player_2: None,
                        player_turn: player_id.to_string(),
                        game_start: false,
                        start_roll: start_roll,
                        game_over: false,
                        game_msg: game_msg,
                    };
                    self.game_state.insert(game_id.to_string(), game_state);

                    println!("GAME STATE: {:?}", self.game_state);
                } else {
                    println!("GAME ERROR, CLIENT RECONNECTED AFTER SERVER CLOSED");
                }
            }
        };
    }

    pub async fn connect(
        &mut self,
        tx: mpsc::UnboundedSender<Msg>,
        game_id: String,
        player_id: Uuid,
    ) -> PlayerId {
        self.sessions.insert(player_id, tx);

        println!("{:?} connected to game_id: {:?}", player_id, game_id);

        let game_id_clone = game_id.clone();
        let game_id_clone2 = game_id.clone();

        self.game_id
            .entry(game_id)
            .or_insert_with(HashSet::new)
            .insert(player_id);

        match self.game_state.get(&game_id_clone) {
            Some(_) => {
                if let Some(game_state) =
                    self.game_state.iter().find_map(|(game_id, game_state)| {
                        game_id.contains(&game_id_clone).then_some(game_state)
                    })
                {
                    if game_state.game_start == false && game_state.player_1 != player_id {
                        self.game_state
                            .entry(game_id_clone)
                            .and_modify(|game_state| {
                                game_state.player_2 = Some(player_id);
                                game_state.player_count.fetch_add(1, Ordering::SeqCst);
                                if game_state.player_count.load(Ordering::SeqCst) == 2 {
                                    let msg = format!("\u{1f9df} has joined the game");
                                    game_state.game_msg.roll_msg.push(msg);

                                    game_state.game_start = true;
                                }
                            });
                        self.update_game_feed(&game_id_clone2);

                        self.send_status_message(player_id, format!("player_two_icon"));
                    } else {
                        if game_state.player_1 == player_id {
                            self.send_status_message(player_id, format!("reconn"));
                        } else if game_state.player_2.unwrap() == player_id {
                            self.send_status_message(player_id, format!("reconn"));

                            self.send_status_message(player_id, format!("player_two_icon"));
                        } else {
                            self.send_status_message(player_id, format!("spec"));
                        }
                    }
                }
            }

            None => {}
        }

        player_id
    }

    pub async fn disconnect(&mut self, player_id: PlayerId, game_id: GameId) {
        println!("{:?} disconnected from game_id: {:?}", player_id, game_id);

        if self.sessions.remove(&player_id).is_some() {
            for (_name, sessions) in &mut self.game_id {
                sessions.remove(&player_id);
            }
        }
    }

    pub async fn run(mut self) -> io::Result<()> {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                Command::Connect {
                    player_tx,
                    game_id,
                    player_id,
                } => {
                    self.connect(player_tx, game_id, player_id).await;
                }

                Command::Disconnect { player_id, game_id } => {
                    self.disconnect(player_id, game_id).await;
                }

                Command::Message {
                    player_id,
                    msg,
                    game_id,
                } => {
                    self.new_turn(player_id, msg.to_string(), game_id).await;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GameServerHandle {
    pub cmd_tx: mpsc::UnboundedSender<Command>,
}

impl GameServerHandle {
    pub async fn handle_connect(
        &self,
        player_tx: mpsc::UnboundedSender<String>,
        game_id: String,
        player_id: PlayerId,
    ) {
        self.cmd_tx
            .send(Command::Connect {
                player_tx,
                game_id,
                player_id,
            })
            .unwrap();
    }

    pub async fn handle_send(&self, player_id: PlayerId, msg: impl Into<String>, game_id: GameId) {
        self.cmd_tx
            .send(Command::Message {
                msg: msg.into(),
                player_id,
                game_id,
            })
            .unwrap();
    }

    pub fn handle_disconnect(&self, player_id: PlayerId, game_id: GameId) {
        self.cmd_tx
            .send(Command::Disconnect { player_id, game_id })
            .unwrap();
    }
}

async fn roll_die(num: u32) -> u32 {
    let mut rng = rand::thread_rng();

    rng.gen_range(1..=num)
}
