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
        keep_alive_tx: oneshot::Sender<PlayerId>,
        game_id: String,
    },

    Disconnect {
        player_id: PlayerId,
    },

    Message {
        msg: Msg,
        player_id: PlayerId,
        keep_alive_tx: oneshot::Sender<()>,
    },
}

#[derive(Debug)]
pub struct GameState {
    roll: u32,
    player_count: Arc<AtomicUsize>,
    player_1: String,
    player_2: Option<String>,
    player_turn: String,
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

    pub async fn send_game_message(&self, arena: &str, msg: impl Into<String>) {
        let msg = msg.into();

        if let Some(sessions) = self.game_arena.get(arena) {
            for player_id in sessions {
                if let Some(cmd_tx) = self.sessions.get(player_id) {
                    let _ = cmd_tx.send(msg.clone());
                }
            }
        }
    }

    pub async fn send_roll(&mut self, player_id: PlayerId, roll: String) {
        if let Some(arena) = self
            .game_arena
            .iter()
            .find_map(|(arena, players)| players.contains(&player_id).then_some(arena))
        {
            match self.game_state.get_mut(arena) {
                Some(_) => {
                    if let Some(game_state) = self
                        .game_state
                        .iter()
                        .find_map(|(arena, game_state)| arena.contains(arena).then_some(game_state))
                    {
                        if game_state.player_turn == player_id.to_string() {
                            if game_state.player_1 == player_id.to_string()
                                && game_state.player_count.load(Ordering::SeqCst) >= 2
                            {
                                let roll = roll_die(game_state.roll).await;

                                self.send_game_message(arena, roll.to_string()).await;
                                let new_turn = self.game_state.get_mut(arena).unwrap();

                                new_turn.roll = roll;

                                if let Some(player_2) = &new_turn.player_2 {
                                    new_turn.player_turn = player_2.to_string()
                                }

                                println!("Arena Score: {:?}", new_turn);
                            } else if game_state.player_2 == Some(player_id.to_string())
                                && game_state.player_count.load(Ordering::SeqCst) >= 2
                            {
                                let roll = roll_die(game_state.roll).await;

                                self.send_game_message(arena, roll.to_string()).await;
                                let new_turn = self.game_state.get_mut(arena).unwrap();

                                new_turn.roll = roll;

                                new_turn.player_turn = new_turn.player_1.clone();

                                println!("Arena Score: {:?}", new_turn);
                            } else {
                                if game_state.player_count.load(Ordering::SeqCst) == 1 {
                                    println!("waiting for player 2")
                                } else if game_state.player_count.load(Ordering::SeqCst) > 2 {
                                    println!("the arena is full")
                                } else {
                                    println!("its not your turn!!")
                                }
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
                        player_1: player_id.to_string(),
                        player_2: None,
                        player_turn: player_id.to_string(),
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

        match self.game_state.get_mut(&game_id) {
            Some(_) => {
                if let Some(game_state) = self
                    .game_state
                    .iter()
                    .find_map(|(arena, game_state)| arena.contains(arena).then_some(game_state))
                {
                    if game_state.player_count.load(Ordering::SeqCst) == 1 {
                        let new_turn = self.game_state.get_mut(&game_id).unwrap();

                        new_turn.player_count.fetch_add(1, Ordering::SeqCst);
                        new_turn.player_2 = Some(player_id.to_string());

                        println!("new player joined the arena: {:?}", new_turn);
                    } else {
                        let new_turn = self.game_state.get_mut(&game_id).unwrap();

                        new_turn.player_count.fetch_add(1, Ordering::SeqCst);
                        println!("new spectator joined the arena: {:?}", new_turn);
                    }
                }
            }
            None => {
                println!("no game exists, creating game");
            }
        };

        self.game_arena
            .entry(game_id)
            .or_insert_with(HashSet::new)
            .insert(player_id);

        player_id
    }

    pub async fn disconnect(&mut self, player_id: PlayerId) {
        println!("{:?} disconnected", player_id);

        let mut game_arena: Vec<String> = Vec::new();

        if self.sessions.remove(&player_id).is_some() {
            for (name, sessions) in &mut self.game_arena {
                if sessions.remove(&player_id) {
                    game_arena.push(name.to_owned());
                }
            }
        }

        for game in game_arena {
            self.send_game_message(&game, format!("{player_id} has left the game"))
                .await;
        }
    }

    pub async fn run(mut self) -> io::Result<()> {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                Command::Connect {
                    player_tx,
                    keep_alive_tx,
                    game_id,
                } => {
                    let player_id = self.connect(player_tx, game_id).await;
                    let _ = keep_alive_tx.send(player_id);
                }

                Command::Disconnect { player_id } => {
                    self.disconnect(player_id).await;
                }

                Command::Message {
                    player_id,
                    msg,
                    keep_alive_tx,
                } => {
                    self.send_roll(player_id, msg.to_string()).await;
                    let _ = keep_alive_tx.send(());
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
        let (keep_alive_tx, keep_alive_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::Connect {
                player_tx,
                keep_alive_tx,
                game_id,
            })
            .unwrap();

        keep_alive_rx.await.unwrap()
    }

    pub async fn handle_send(&self, player_id: PlayerId, msg: impl Into<String>) {
        let (keep_alive_tx, keep_alive_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::Message {
                msg: msg.into(),
                player_id,
                keep_alive_tx,
            })
            .unwrap();

        keep_alive_rx.await.unwrap();
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
