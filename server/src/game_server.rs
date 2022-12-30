use std::{
    collections::{HashMap, HashSet},
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use rand::Rng;
use regex::Regex;
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

#[derive(Debug)]
pub struct GameState {
    roll: u32,
    player_count: Arc<AtomicUsize>,
    player_1: Uuid,
    player_2: Option<Uuid>,
    player_turn: String,
    game_start: bool,
    start_roll: u32,
}

#[derive(Debug)]
pub struct GameServer {
    sessions: HashMap<PlayerId, mpsc::UnboundedSender<Msg>>,
    game_arena: HashMap<GameId, HashSet<PlayerId>>,
    cmd_rx: mpsc::UnboundedReceiver<Command>,
    game_state: HashMap<GameId, GameState>,
}
impl GameServer {
    pub fn new() -> (Self, GameServerHandle) {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        (
            Self {
                sessions: HashMap::new(),
                game_arena: HashMap::new(),
                cmd_rx,
                game_state: HashMap::new(),
            },
            GameServerHandle { cmd_tx },
        )
    }

    pub async fn send_game_message(
        &self,
        game_id: &str,
        send_all: bool,
        player_id: PlayerId,
        msg: impl Into<String>,
    ) {
        let msg = msg.into();

        if send_all == true {
            if let Some(arena) = self.game_arena.get(game_id) {
                for player_ids in arena {
                    if let Some(cmd_tx) = self.sessions.get(player_ids) {
                        let _ = cmd_tx.send(msg.clone());
                    }
                }
            }
        } else {
            if let Some(cmd_tx) = self.sessions.get(&player_id) {
                let _ = cmd_tx.send(msg.clone());
            }
        }
    }

    pub async fn new_turn(&mut self, player_id: PlayerId, roll: String, arena: GameId) {
        let p1 = "\u{1F9D9}\u{200D}\u{2642}\u{FE0F}";
        let p2 = "\u{1F9DF}";
        match self.game_state.get(&arena) {
            Some(_) => {
                // let arena = arena.clone();
                if let Some(game_state) = self
                    .game_state
                    .iter()
                    .find_map(|(game, game_state)| game.contains(&arena).then_some(game_state))
                {
                    if game_state.player_turn == player_id.to_string()
                        && game_state.game_start == true
                    {
                        let re = Regex::new(r"\d").unwrap();

                        let contains_number = re.is_match(&roll);

                        if game_state.player_1 == player_id && contains_number == false {
                            let roll = roll_die(game_state.roll).await;
                            if roll != 1 {
                                let msg = format!("{p1} {roll}");
                                let send_all = true;
                                self.send_game_message(
                                    &arena,
                                    send_all,
                                    player_id,
                                    msg.to_string(),
                                )
                                .await;

                                self.game_state
                                    .entry(arena.clone())
                                    .and_modify(|game_state| {
                                        game_state.roll = roll;
                                        if let Some(player_2) = game_state.player_2.clone() {
                                            game_state.player_turn = player_2.to_string()
                                        }
                                    });
                            } else {
                                let msg = format!(
                                    "{p1} 1 \u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}"
                                );
                                let send_all = true;
                                self.send_game_message(
                                    &arena,
                                    send_all,
                                    player_id,
                                    msg.to_string(),
                                )
                                .await;
                                //send defeat update to player 1
                                let msg = format!(
                                    "\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480} YOU DIED!!!"
                                );
                                let send_all = false;
                                self.send_game_message(
                                    &arena,
                                    send_all,
                                    player_id,
                                    msg.to_string(),
                                )
                                .await;
                                //send victory message to player 2
                                let msg = format!(
                                    "\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6} YOU WON!!!"
                                );
                                let send_all = false;
                                if let Some(player_2) = game_state.player_2.clone() {
                                    self.send_game_message(
                                        &arena,
                                        send_all,
                                        player_2,
                                        msg.to_string(),
                                    )
                                    .await;
                                }

                                self.game_state
                                    .entry(arena.clone())
                                    .and_modify(|game_state| {
                                        game_state.roll = roll;
                                        game_state.game_start = false;
                                    });
                            }
                        } else if game_state.player_2 == Some(player_id) && contains_number == false
                        {
                            let roll = roll_die(game_state.roll).await;
                            if roll != 1 {
                                let msg = format!("{p2} {roll}");
                                let send_all = true;
                                self.send_game_message(
                                    &arena,
                                    send_all,
                                    player_id,
                                    msg.to_string(),
                                )
                                .await;
                                self.game_state
                                    .entry(arena.clone())
                                    .and_modify(|game_state| {
                                        game_state.roll = roll;
                                        game_state.player_turn = game_state.player_1.to_string()
                                    });
                            } else {
                                let msg = format!(
                                    "{p2} 1 \u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}"
                                );
                                let send_all = true;
                                self.send_game_message(
                                    &arena,
                                    send_all,
                                    player_id,
                                    msg.to_string(),
                                )
                                .await;
                                //send defeat update to player 2
                                let msg = format!(
                                    "\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480} YOU DIED!!!"
                                );
                                let send_all = false;
                                self.send_game_message(
                                    &arena,
                                    send_all,
                                    player_id,
                                    msg.to_string(),
                                )
                                .await;
                                // send victory message to player 1
                                let msg = format!(
                                    "\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6} YOU WON!!!"
                                );
                                let send_all = false;
                                let player_id = game_state.player_1;
                                self.send_game_message(
                                    &arena,
                                    send_all,
                                    player_id,
                                    msg.to_string(),
                                )
                                .await;
                                self.game_state
                                    .entry(arena.clone())
                                    .and_modify(|game_state| {
                                        game_state.roll = roll;
                                        game_state.game_start = false;
                                    });
                            }
                        }
                    } else {
                        if game_state.game_start == false && game_state.roll != 1 {
                            let msg = format!("waiting for {p2} to join...");
                            let send_all = false;
                            self.send_game_message(&arena, send_all, player_id, msg.to_string())
                                .await;
                        } else if game_state.game_start == false && game_state.roll == 1 {
                            let msg = "GameOver!";
                            let send_all = true;
                            self.send_game_message(&arena, send_all, player_id, msg.to_string())
                                .await;
                        } else if game_state.game_start == true
                            && game_state.roll == game_state.start_roll
                        {
                            if Some(player_id) == game_state.player_2 {
                                let msg = format!("waiting for {p1} to start the game...");
                                let send_all = false;
                                self.send_game_message(
                                    &arena,
                                    send_all,
                                    player_id,
                                    msg.to_string(),
                                )
                                .await;
                            }

                            let player_id = game_state.player_1;

                            let msg = format!("roll to start the game...",);
                            let send_all = false;
                            self.send_game_message(&arena, send_all, player_id, msg.to_string())
                                .await;
                        } else {
                            let msg = "It's not your turn!";
                            let send_all = false;
                            self.send_game_message(&arena, send_all, player_id, msg.to_string())
                                .await;
                        }
                    }
                }
            }

            None => {
                let start_roll: u32 = match roll.trim().parse::<u32>() {
                    Ok(parsed_input) => parsed_input,

                    Err(_) => 1,
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
                    };
                    self.game_state.insert(arena.to_string(), game_state);

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

        println!("{:?} connected to arena: {:?}", player_id, game_id);

        let game_id_clone = game_id.clone();
        let game_id_clone2 = game_id.clone();

        self.game_arena
            .entry(game_id)
            .or_insert_with(HashSet::new)
            .insert(player_id);

        match self.game_state.get(&game_id_clone) {
            Some(_) => {
                if let Some(game_state) = self.game_state.iter().find_map(|(arena, game_state)| {
                    arena.contains(&game_id_clone).then_some(game_state)
                }) {
                    if game_state.game_start == false && game_state.player_1 != player_id {
                        self.game_state
                            .entry(game_id_clone)
                            .and_modify(|game_state| {
                                game_state.player_2 = Some(player_id);
                                game_state.player_count.fetch_add(1, Ordering::SeqCst);
                                if game_state.player_count.load(Ordering::SeqCst) == 2 {
                                    game_state.game_start = true;
                                }
                            });
                        let send_all = true;
                        self.send_game_message(
                            &game_id_clone2,
                            send_all,
                            player_id,
                            format!("\u{1f9df} has joined the game"),
                        )
                        .await;
                        let send_all = false;
                        self.send_game_message(
                            &game_id_clone2,
                            send_all,
                            player_id,
                            format!("player_icon_set"),
                        )
                        .await;
                    } else {
                        if game_state.player_1 == player_id {
                            let send_all = true;
                            self.send_game_message(
                                &game_id_clone,
                                send_all,
                                player_id,
                                format!("\u{1F9D9}\u{200D}\u{2642}\u{FE0F} has joined the game"),
                            )
                            .await;
                        } else if game_state.player_2.unwrap() == player_id {
                            let send_all = true;
                            self.send_game_message(
                                &game_id_clone,
                                send_all,
                                player_id,
                                format!("\u{1f9df} has joined the game"),
                            )
                            .await;
                            let send_all = false;
                            self.send_game_message(
                                &game_id_clone2,
                                send_all,
                                player_id,
                                format!("player_icon_set"),
                            )
                            .await;
                        } else {
                            let send_all = false;
                            self.send_game_message(
                                &game_id_clone2,
                                send_all,
                                player_id,
                                format!("spectator"),
                            )
                            .await;
                        }
                    }
                }
            }

            None => {}
        }

        player_id
    }

    pub async fn disconnect(&mut self, player_id: PlayerId, game_id: GameId) {
        println!("{:?} disconnected from arena: {:?}", player_id, game_id);
        let game_id_clone = game_id.clone();
        match self.game_state.get(&game_id) {
            Some(_) => {
                if let Some(game_state) = self
                    .game_state
                    .iter()
                    .find_map(|(arena, game_state)| arena.contains(&game_id).then_some(game_state))
                {
                    if game_state.player_1 == player_id {
                        let send_all = true;
                        self.send_game_message(
                            &game_id_clone,
                            send_all,
                            player_id,
                            format!("\u{1F9D9}\u{200D}\u{2642}\u{FE0F} has left the game"),
                        )
                        .await;
                    } else {
                        let send_all = true;
                        self.send_game_message(
                            &game_id_clone,
                            send_all,
                            player_id,
                            format!("\u{1f9df} has left the game"),
                        )
                        .await;
                    }
                }
            }

            None => {}
        }

        if self.sessions.remove(&player_id).is_some() {
            for (_name, sessions) in &mut self.game_arena {
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
