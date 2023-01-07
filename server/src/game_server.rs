use crate::SharedState;
use rand::Rng;
use serde::{Deserialize, Serialize};

use std::{
    collections::{HashMap, HashSet},
    io,
};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

pub type PlayerId = Uuid;
pub type GameId = String;
pub type Msg = String;

#[derive(Serialize, Clone, Deserialize)]
pub enum GameMessage {
    Spectate,
    StartGame(String),
    Reconnect,
    NoGameFound,
    P1Join,
    P2Join,
    Status(String),
    GameScore(GameScore),
    StartRoll(String),
    Pong,
}

#[derive(Debug)]
pub enum Command {
    Connect {
        player_tx: mpsc::UnboundedSender<Msg>,
        game_id: GameId,
        player_id: PlayerId,
        state: SharedState,
    },

    Disconnect {
        player_id: PlayerId,
        game_id: GameId,
    },

    Turn {
        player_id: PlayerId,
        game_id: GameId,
    },
}

#[derive(Debug, Clone)]
pub struct GameServerHandle {
    pub server_tx: mpsc::UnboundedSender<Command>,
}

impl GameServerHandle {
    pub async fn handle_connect(
        &self,
        player_tx: mpsc::UnboundedSender<String>,
        game_id: String,
        player_id: PlayerId,
        state: SharedState,
    ) {
        self.server_tx
            .send(Command::Connect {
                player_tx,
                game_id,
                player_id,
                state,
            })
            .unwrap();
    }

    pub async fn handle_send(&self, player_id: PlayerId, game_id: GameId) {
        self.server_tx
            .send(Command::Turn { player_id, game_id })
            .unwrap();
    }

    pub fn handle_disconnect(&self, player_id: PlayerId, game_id: GameId) {
        self.server_tx
            .send(Command::Disconnect { player_id, game_id })
            .unwrap();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameScore {
    client_feed: Vec<String>,
}

#[derive(Debug)]
pub struct GameState {
    roll: u32,
    player_1: Uuid,
    player_2: Option<Uuid>,
    player_turn: String,
    game_start: bool,
    start_roll: u32,
    game_over: bool,
    game_score: GameScore,
}

#[derive(Debug)]
pub struct GameServer {
    sessions: HashMap<PlayerId, Vec<mpsc::UnboundedSender<Msg>>>,
    players: HashMap<GameId, HashSet<PlayerId>>,
    server_rx: mpsc::UnboundedReceiver<Command>,
    game_rooms: HashMap<GameId, GameState>,
}
impl GameServer {
    pub fn new() -> (Self, GameServerHandle) {
        let (server_tx, server_rx) = mpsc::unbounded_channel();

        (
            Self {
                sessions: HashMap::new(),
                players: HashMap::new(),
                server_rx,
                game_rooms: HashMap::new(),
            },
            GameServerHandle { server_tx },
        )
    }

    pub async fn run(mut self) -> io::Result<()> {
        while let Some(cmd) = self.server_rx.recv().await {
            match cmd {
                Command::Connect {
                    player_tx,
                    game_id,
                    player_id,
                    state,
                } => {
                    self.connect(player_tx, game_id, player_id, state).await;
                }

                Command::Disconnect { player_id, game_id } => {
                    self.disconnect(player_id, game_id).await;
                }

                Command::Turn { player_id, game_id } => {
                    self.new_turn(player_id, game_id).await;
                }
            }
        }

        Ok(())
    }

    async fn update_game_feed(&self, game_id: &str) {
        let game = self.game_rooms.get(game_id).unwrap();
        let msg = GameMessage::GameScore(game.game_score.clone());

        if let Some(game_id) = self.players.get(game_id) {
            for player_ids in game_id {
                if let Some(cmd_tx) = self.sessions.get(player_ids) {
                    for tx in cmd_tx {
                        let _ = tx.send(serde_json::to_string(&msg).unwrap());
                    }
                }
            }
        }
    }

    async fn send_to_other(&self, game_id: &str, msg: GameMessage, player_id: Uuid) {
        if let Some(game_id) = self.players.get(game_id) {
            for player_ids in game_id {
                if *player_ids != player_id {
                    if let Some(cmd_tx) = self.sessions.get(player_ids) {
                        for tx in cmd_tx {
                            let _ = tx.send(serde_json::to_string(&msg).unwrap());
                        }
                    }
                }
            }
        }
    }

    async fn send_status_message(&self, player_id: PlayerId, msg: GameMessage) {
        if let Some(cmd_tx) = self.sessions.get(&player_id) {
            for tx in cmd_tx {
                let _ = tx.send(serde_json::to_string(&msg).unwrap());
            }
        }
    }

    async fn new_turn(&mut self, player_id: PlayerId, game_id: GameId) {
        let p1 = "\u{1F9D9}\u{200D}\u{2642}\u{FE0F}";
        let p2 = "\u{1F9DF}";

        if self.game_rooms.get(&game_id).is_some() {
            if let Some(game_state) = self
                .game_rooms
                .iter()
                .find_map(|(game, game_state)| game.contains(&game_id).then_some(game_state))
            {
                {
                    if game_state.player_turn == player_id.to_string()
                        && !game_state.game_over
                        && game_state.game_start
                    {
                        let roll_between = game_state.roll;
                        let roll = roll_die(game_state.roll).await;
                        if roll != 1 {
                            //handle player 1 turn
                            if player_id == game_state.player_1 {
                                let msg = format!("{p1} {roll} \u{1F3B2} (1-{roll_between})");
                                self.game_rooms
                                    .entry(game_id.clone())
                                    .and_modify(|game_state| {
                                        game_state.roll = roll;
                                        game_state.game_score.client_feed.push(msg);

                                        if let Some(player_2) = game_state.player_2 {
                                            game_state.player_turn = player_2.to_string();
                                        }
                                    });
                                let status_msg =
                                    GameMessage::Status(format!("{p1} \u{1F3B2} {roll}"));

                                self.send_status_message(player_id, status_msg).await;
                                let status_msg =
                                    GameMessage::Status(format!("{p2} \u{1F3B2} It's your roll!"));
                                self.send_to_other(&game_id, status_msg, player_id).await;
                                //handle player 2 turn
                            } else if Some(player_id) == game_state.player_2 {
                                let msg = format!("{p2} {roll} \u{1F3B2} (1-{roll_between})");
                                self.game_rooms
                                    .entry(game_id.clone())
                                    .and_modify(|game_state| {
                                        game_state.roll = roll;
                                        game_state.game_score.client_feed.push(msg);
                                        game_state.player_turn = game_state.player_1.to_string()
                                    });
                                let status_msg =
                                    GameMessage::Status(format!("{p2} \u{1F3B2} {roll}"));
                                //send player roll as status update
                                self.send_status_message(player_id, status_msg).await;
                                let status_msg =
                                    GameMessage::Status(format!("{p1} \u{1F3B2} It's your roll!"));
                                self.send_to_other(&game_id, status_msg, player_id).await;
                            }
                            self.update_game_feed(&game_id).await;
                        } else {
                            let defeat1 = GameMessage::Status(format!("{p1} \u{1F480}"));
                            let victory1 = GameMessage::Status(format!("{p1} \u{1F3C6}"));
                            let defeat2 = GameMessage::Status(format!("{p2} \u{1F480}"));
                            let victory2 = GameMessage::Status(format!("{p2} \u{1F3C6}"));
                            //handle player 1 death
                            if player_id == game_state.player_1 {
                                //send victory status message to player 2
                                let player2 = Some(game_state.player_2).unwrap();
                                self.send_status_message(player2.unwrap(), victory2).await;
                                //send defeat status message to player 1
                                let player1 = game_state.player_1;

                                self.send_status_message(player1, defeat1).await;
                                //deathroll feed update
                                let msg = format!(
                                        "{p1} 1 \u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480} \u{1F3B2} (1-{roll_between})");
                                self.game_rooms
                                    .entry(game_id.clone())
                                    .and_modify(|game_state| {
                                        game_state.roll = roll;
                                        game_state.game_score.client_feed.push(msg);
                                        game_state.game_over = true;
                                    });
                                //handle player 1 death
                            } else if Some(player_id) == game_state.player_2 {
                                //send victory message to player 1
                                let player1 = game_state.player_1;
                                self.send_status_message(player1, victory1).await;
                                //send defeat status message to player 2
                                let player2 = Some(game_state.player_2).unwrap();
                                self.send_status_message(player2.unwrap(), defeat2).await;
                                //deathroll feed update
                                let msg = format!(
                                        "{p2} 1 \u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480} \u{1F3B2} (1-{roll_between})");
                                self.game_rooms
                                    .entry(game_id.clone())
                                    .and_modify(|game_state| {
                                        game_state.roll = roll;
                                        game_state.game_score.client_feed.push(msg);
                                        game_state.game_over = true;
                                    });
                            }

                            self.update_game_feed(&game_id).await;
                        }
                    } else if !game_state.game_start {
                        self.game_rooms
                            .entry(game_id.clone())
                            .and_modify(|game_state| {
                                game_state.game_start = true;
                            });

                        let msg =
                            GameMessage::StartGame(format!("{p2} \u{1F3B2} waiting for {p1}"));
                        self.send_status_message(player_id, msg).await;

                        let msg = GameMessage::StartGame(format!("{p1} \u{1F3B2} /roll to start"));
                        self.send_to_other(&game_id, msg, player_id).await;
                    } else if game_state.game_over {
                        //game over so do nothing
                    } else if game_state.game_start && game_state.roll == game_state.start_roll {
                    } else {
                        //not players turn so do nothing
                    }
                }
            }
        }
    }

    async fn connect(
        &mut self,
        tx: mpsc::UnboundedSender<Msg>,
        game_id: String,
        player_id: Uuid,
        state: SharedState,
    ) -> PlayerId {
        let p1 = "\u{1F9D9}\u{200D}\u{2642}\u{FE0F}";
        let p2 = "\u{1F9DF}";

        if let Some(value) = self.sessions.get_mut(&player_id) {
            // If session exists then push new tx to vec (fix opening multiple tabs of the same game)
            value.push(tx);
        } else {
            let tx_vec = vec![tx];
            self.sessions.insert(player_id, tx_vec);
        }

        //println!("{:?} connected to game_id: {:?}", player_id, game_id);

        let game_id_clone = game_id.clone();
        let game_id_clone2 = game_id.clone();

        self.players
            .entry(game_id)
            .or_insert_with(HashSet::new)
            .insert(player_id);

        match self.game_rooms.get(&game_id_clone) {
            Some(_) => {
                if let Some(game_state) =
                    self.game_rooms.iter().find_map(|(game_id, game_state)| {
                        game_id.contains(&game_id_clone).then_some(game_state)
                    })
                {
                    if !game_state.game_start && game_state.player_1 != player_id {
                        //send p2 join message to show join screen
                        self.send_status_message(player_id, GameMessage::P2Join)
                            .await;
                        //display start roll
                        let start_roll = game_state.start_roll;
                        self.send_status_message(
                            player_id,
                            GameMessage::StartRoll(start_roll.to_string()),
                        )
                        .await;
                        //add player as p2
                        self.game_rooms
                            .entry(game_id_clone)
                            .and_modify(|game_state| {
                                game_state.player_2 = Some(player_id);
                            });
                    } else if !game_state.game_start && game_state.player_1 == player_id {
                        let start_roll = game_state.start_roll;
                        self.send_status_message(
                            player_id,
                            GameMessage::StartRoll(start_roll.to_string()),
                        )
                        .await;
                    } else {
                        if game_state.player_1 == player_id && game_state.game_start {
                            let msg = GameMessage::Status(format!("{p1} \u{1F3B2}"));
                            self.send_status_message(player_id, msg).await;
                        } else if game_state.player_2.unwrap() == player_id && game_state.game_start
                        {
                            let msg = GameMessage::Status(format!("{p2} \u{1F3B2}"));
                            self.send_status_message(player_id, msg).await;
                        } else {
                            self.send_status_message(player_id, GameMessage::Spectate)
                                .await;
                        }
                        self.update_game_feed(&game_id_clone2).await;
                        self.send_status_message(player_id, GameMessage::Reconnect)
                            .await;
                        let start_roll = game_state.start_roll;
                        self.send_status_message(
                            player_id,
                            GameMessage::StartRoll(start_roll.to_string()),
                        )
                        .await;
                    }
                }
            }

            None => {
                let start_roll = state
                    .read()
                    .unwrap()
                    .start_roll
                    .get(&game_id_clone)
                    .map(|s| s.trim().parse::<u32>().unwrap_or_default())
                    .unwrap_or_default();

                sleep(Duration::from_millis(300)).await;

                //if start roll contains the game_id then make a new game, if not redirect to 404
                if start_roll != 0 {
                    let game_score = GameScore {
                        client_feed: Vec::new(),
                    };

                    let game_state = GameState {
                        roll: start_roll,
                        player_1: player_id,
                        player_2: None,
                        player_turn: player_id.to_string(),
                        game_start: false,
                        start_roll,
                        game_over: false,
                        game_score,
                    };

                    self.game_rooms
                        .insert(game_id_clone.to_string(), game_state);

                    println!("NEW GAME ADDED {:?}", game_id_clone);

                    self.send_status_message(player_id, GameMessage::P1Join)
                        .await;
                    //display start roll

                    self.send_status_message(
                        player_id,
                        GameMessage::StartRoll(start_roll.to_string()),
                    )
                    .await;
                } else {
                    self.send_status_message(player_id, GameMessage::NoGameFound)
                        .await;
                }
            }
        }

        player_id
    }

    async fn disconnect(&mut self, player_id: PlayerId, _game_id: GameId) {
        //println!("{:?} disconnected from game_id: {:?}", player_id, game_id);

        if self.sessions.remove(&player_id).is_some() {
            for sessions in self.players.values_mut() {
                sessions.remove(&player_id);
            }
        }
    }
}

async fn roll_die(num: u32) -> u32 {
    let mut rng = rand::thread_rng();

    rng.gen_range(1..=num)
}
