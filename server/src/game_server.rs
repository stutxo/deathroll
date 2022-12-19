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

use crate::{GameId, Msg, PlayerId};

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
}

#[derive(Debug)]
pub struct GameServer {
    sessions: HashMap<PlayerId, mpsc::UnboundedSender<Msg>>,
    game_arena: HashMap<GameId, HashSet<PlayerId>>,
    cmd_rx: mpsc::UnboundedReceiver<Command>,
    roll_amount: HashMap<GameId, GameState>,
}
impl GameServer {
    pub fn new() -> (Self, GameServerHandle) {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        (
            Self {
                sessions: HashMap::new(),
                game_arena: HashMap::new(),
                cmd_rx,
                roll_amount: HashMap::new(),
            },
            GameServerHandle { cmd_tx },
        )
    }

    pub async fn send_system_message(&self, room: &str, msg: impl Into<String>) {
        let msg = msg.into();

        if let Some(sessions) = self.game_arena.get(room) {
            for player_id in sessions {
                if let Some(tx) = self.sessions.get(player_id) {
                    let _ = tx.send(msg.clone());
                }
            }
        }
    }

    pub async fn send_message(&mut self, player_id: PlayerId, msg: impl Into<String>) {
        let msg = msg.into();
        let msg_clone = msg.clone();

        if let Some(room) = self
            .game_arena
            .iter()
            .find_map(|(room, participants)| participants.contains(&player_id).then_some(room))
        {
            self.send_system_message(room, msg).await;
            let new_roll = self.roll_amount.get_mut(room).unwrap();

            let roll: String = msg_clone.into();
            let roll: u32 = match roll.trim().parse::<u32>() {
                Ok(parsed_input) => parsed_input,

                Err(_) => 1,
            };
            new_roll.player_count.fetch_add(1, Ordering::SeqCst);
            new_roll.roll = roll;

            println!("{:?}", self.roll_amount);
        };
    }

    pub async fn connect(&mut self, tx: mpsc::UnboundedSender<Msg>, game_id: String) -> PlayerId {
        let player_id = Uuid::new_v4();
        self.sessions.insert(player_id, tx);

        self.game_arena
            .entry(game_id)
            .or_insert_with(HashSet::new)
            .insert(player_id);

        if let Some(game_state) = self
            .roll_amount
            .iter()
            .find_map(|(room, game_state)| room.contains(room).then_some(game_state))
        {
            
        }

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
            self.send_system_message(&game, format!("{player_id} has left the game"))
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
                    if let Some(room) = self.game_arena.iter().find_map(|(room, participants)| {
                        participants.contains(&player_id).then_some(room)
                    }) {
                        match self.roll_amount.get_mut(room) {
                            Some(_) => {
                                if let Some(game_state) =
                                    self.roll_amount.iter().find_map(|(room, game_state)| {
                                        room.contains(room).then_some(game_state)
                                    })
                                {
                                    let roll = roll_die(game_state.roll).await;
                                    self.send_message(player_id, roll.to_string()).await;
                                    let _ = keep_alive_tx.send(());
                                }
                            }
                            None => {
                                let start_roll: u32 = match msg.trim().parse::<u32>() {
                                    Ok(parsed_input) => parsed_input,

                                    Err(_) => 1,
                                };

                                let game_state = GameState {
                                    roll: start_roll,
                                    player_count: Arc::new(AtomicUsize::new(1)),
                                };
                                self.roll_amount.insert(room.to_string(), game_state);

                                println!("{:?}", self.roll_amount);
                                let _ = keep_alive_tx.send(());
                            }
                        };
                    };
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
