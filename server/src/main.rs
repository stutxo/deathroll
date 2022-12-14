use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::routing::SpaRouter;

use futures::lock::Mutex;
use futures::{sink::SinkExt, stream::StreamExt};
use nanoid::nanoid;
use std::collections::HashMap;
use std::{collections::HashSet, net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    tx: broadcast::Sender<String>,
    roll: Mutex<Vec<u32>>,
    arena: Mutex<HashMap<String, String>>,
    arena_full: Mutex<bool>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "websocket=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let spa = SpaRouter::new("/assets", "../dist");

    let (tx, _rx) = broadcast::channel(100);
    let roll: Mutex<Vec<u32>> = Mutex::new(Vec::new());
    let arena = Mutex::new(HashMap::new());
    let arena_full = Mutex::new(false);

    let app_state = Arc::new(AppState {
        tx,
        roll,
        arena,
        arena_full,
    });

    let app = Router::new()
        .merge(spa)
        .route("/:id/ws", get(websocket_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    uri: axum::http::Uri,
) -> impl IntoResponse {
    let arena_id = uri.to_string();
    println!("connected to arena {:?}", arena_id);
    ws.on_upgrade(|socket| websocket(socket, state, arena_id))
}

async fn websocket(stream: WebSocket, state: Arc<AppState>, arena_id: String) {
    println!("{:?}", arena_id);
    let player_id = nanoid!();
    let mut arena = state.arena.lock().await;

    arena.insert(player_id, arena_id);

    drop(arena);

    let arena1 = state.arena.lock().await;
    let mut seen: HashSet<&String> = HashSet::new();
    println!("{:?}", arena1);
    for (key, value) in arena1.iter() {
        if seen.contains(value) {
            println!("{} arena is full ", value);
        } else {
            seen.insert(value);
            println!("{:?}", seen);
        }
    }

    drop(arena1);

    let (mut sender, mut receiver) = stream.split();

    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(start_roll) = message {
            let arena_full = state.arena_full.lock().await;

            if *arena_full == false {
                check_player_id(&state, &start_roll).await;
                drop(arena_full);
                break;
            } else {
                let _ = sender
                    .send(Message::Text(String::from("game room is full")))
                    .await;
                println!("ffaf");
                return;
            }
        }
    }
    let mut rx = state.tx.subscribe();
    let msg = "test".to_string();

    tracing::debug!("{}", msg);
    let _ = state.tx.send(msg);

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let tx = state.tx.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let _ = tx.send(format!("{}", text));
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    let msg = format!("left.");
    tracing::debug!("{}", msg);
    let _ = state.tx.send(msg);
}

async fn check_player_id(state: &AppState, start_roll: &str) {
    let mut roll = state.roll.lock().await;

    let num_input: u32 = match start_roll.trim().parse::<u32>() {
        Ok(parsed_input) => parsed_input,

        Err(_) => 1,
    };
    println!("{:?}", num_input);

    roll.push(num_input);
    drop(roll);
}
