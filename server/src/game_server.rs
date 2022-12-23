use std::{
    collections::{HashMap, HashSet},
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use rand::Rng;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

pub type PlayerId = Uuid;
pub type GameId = String;
pub type Msg = String;

#[derive(Debug)]
pub enum Command {
    Connect {
        player_tx: mpsc::UnboundedSender<Msg>,
        player_id_tx: oneshot::Sender<PlayerId>,
        game_id: String,
    },

    Disconnect {
        player_id: PlayerId,
    },

    Message {
        msg: Msg,
        player_id: PlayerId,
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

    pub async fn new_turn(&mut self, player_id: PlayerId, roll: String) {
        if let Some(arena) = self
            .game_arena
            .iter()
            .find_map(|(arena, players)| players.contains(&player_id).then_some(arena))
        {
            match self.game_state.get(arena) {
                Some(_) => {
                    if let Some(game_state) = self
                        .game_state
                        .iter()
                        .find_map(|(game, game_state)| game.contains(arena).then_some(game_state))
                    {
                        if game_state.player_turn == player_id.to_string()
                            && game_state.game_start == true
                        {
                            if game_state.player_1 == player_id {
                                let roll = roll_die(game_state.roll).await;
                                if roll != 1 {
                                    let send_all = true;
                                    self.send_game_message(
                                        arena,
                                        send_all,
                                        player_id,
                                        roll.to_string(),
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

                                    println!("GAMEUPDATE: {:?}", self.game_state);
                                } else {
                                    //send defeat message to player 1
                                    let msg =
                                        "\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}";
                                    let send_all = false;
                                    self.send_game_message(
                                        arena,
                                        send_all,
                                        player_id,
                                        msg.to_string(),
                                    )
                                    .await;
                                    let msg =
                                        "\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}";
                                    //send victory message to player 2
                                    if let Some(player_2) = game_state.player_2.clone() {
                                        self.send_game_message(
                                            arena,
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
                            } else if game_state.player_2 == Some(player_id) {
                                let roll = roll_die(game_state.roll).await;
                                if roll != 1 {
                                    let send_all = true;
                                    self.send_game_message(
                                        arena,
                                        send_all,
                                        player_id,
                                        roll.to_string(),
                                    )
                                    .await;
                                    self.game_state
                                        .entry(arena.clone())
                                        .and_modify(|game_state| {
                                            game_state.roll = roll;
                                            game_state.player_turn = game_state.player_1.to_string()
                                        });

                                    println!("GAMEUPDATE: {:?}", self.game_state);
                                } else {
                                    //send defeat message to player 2
                                    let msg =
                                        "\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}";
                                    let send_all = false;
                                    self.send_game_message(
                                        arena,
                                        send_all,
                                        player_id,
                                        msg.to_string(),
                                    )
                                    .await;
                                    let msg =
                                        "\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}";
                                    // send victory message to player 1
                                    let player_id = game_state.player_1;
                                    self.send_game_message(
                                        arena,
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
                                let msg = "waiting for player 2";
                                let send_all = false;
                                self.send_game_message(arena, send_all, player_id, msg.to_string())
                                    .await;
                            } else if game_state.game_start == false && game_state.roll == 1 {
                                //game is over, do nothing.
                            } else if game_state.game_start == true
                                && game_state.roll == game_state.start_roll
                            {
                                let msg = format!(
                                    "start roll: {}, waiting for player 1 to roll...",
                                    game_state.start_roll
                                );
                                let send_all = true;
                                self.send_game_message(arena, send_all, player_id, msg.to_string())
                                    .await;
                            } else {
                                let msg = "you can't do that yet!";
                                let send_all = false;
                                self.send_game_message(arena, send_all, player_id, msg.to_string())
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

                    println!("new game: {:?}", self.game_state);
                }
            };
        };
    }

    pub async fn connect(&mut self, tx: mpsc::UnboundedSender<Msg>, game_id: String) -> PlayerId {
        let player_id = Uuid::new_v4();

        self.sessions.insert(player_id, tx);

        let game_id_clone = game_id.clone();

        self.game_arena
            .entry(game_id)
            .or_insert_with(HashSet::new)
            .insert(player_id);

        match self.game_state.get(&game_id_clone) {
            Some(_) => {
                if let Some(game_state) = self.game_state.iter().find_map(|(arena, game_state)| {
                    arena.contains(&game_id_clone).then_some(game_state)
                }) {
                    if game_state.game_start == false {
                        let msg = "new player joined the arena";
                        let send_all = true;
                        self.send_game_message(&game_id_clone, send_all, player_id, msg)
                            .await;
                        self.game_state
                            .entry(game_id_clone)
                            .and_modify(|game_state| {
                                game_state.player_2 = Some(player_id);
                                game_state.player_count.fetch_add(1, Ordering::SeqCst);
                                if game_state.player_count.load(Ordering::SeqCst) == 2 {
                                    game_state.game_start = true;
                                }
                            });
                    } else {
                        let msg = "new spectator joined the arena";
                        let send_all = true;
                        self.send_game_message(&game_id_clone, send_all, player_id, msg)
                            .await;
                    }
                }
            }

            None => {}
        }

        player_id
    }

    pub async fn disconnect(&mut self, player_id: PlayerId) {
        let mut game_arena: Vec<String> = Vec::new();

        if self.sessions.remove(&player_id).is_some() {
            for (name, sessions) in &mut self.game_arena {
                if sessions.remove(&player_id) {
                    game_arena.push(name.to_owned());
                }
            }
        }

        for game in game_arena {
            let send_all = true;
            self.send_game_message(
                &game,
                send_all,
                player_id,
                format!("{player_id} has left the game"),
            )
            .await;
        }
    }

    pub async fn run(mut self) -> io::Result<()> {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                Command::Connect {
                    player_tx,
                    player_id_tx,
                    game_id,
                } => {
                    let player_id = self.connect(player_tx, game_id).await;
                    let _ = player_id_tx.send(player_id);
                }

                Command::Disconnect { player_id } => {
                    self.disconnect(player_id).await;
                }

                Command::Message { player_id, msg } => {
                    self.new_turn(player_id, msg.to_string()).await;
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
    ) -> PlayerId {
        let (player_id_tx, player_id_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::Connect {
                player_tx,
                player_id_tx,
                game_id,
            })
            .unwrap();

        player_id_rx.await.unwrap()
    }

    pub async fn handle_send(&self, player_id: PlayerId, msg: impl Into<String>) {
        self.cmd_tx
            .send(Command::Message {
                msg: msg.into(),
                player_id,
            })
            .unwrap();
    }

    pub fn handle_disconnect(&self, player_id: PlayerId) {
        self.cmd_tx.send(Command::Disconnect { player_id }).unwrap();
    }
}

async fn roll_die(num: u32) -> u32 {
    let mut rng = rand::thread_rng();

    let points = rng.gen_range(1..=num);

    points
}
