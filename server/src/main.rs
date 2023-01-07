use axum::{
    extract::{ws::WebSocketUpgrade, Path, State},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use axum_extra::routing::SpaRouter;
use game_server::{GameServer, GameServerHandle};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};

use uuid::Uuid;

use websockets::handle_socket;
mod game_server;
mod websockets;

const COOKIE_NAME: &str = "deathroll";

type SharedState = Arc<RwLock<StartRoll>>;

#[derive(Default, Debug)]
pub struct StartRoll {
    start_roll: HashMap<String, String>,
}

#[tokio::main]
async fn main() {
    let (game_server, server_tx) = GameServer::new();
    let run_game = tokio::spawn(game_server.run());

    let shared_state = SharedState::default();

    let spa = SpaRouter::new("/assets", "../dist");

    let app = Router::new()
        .merge(spa)
        .route("/ws/:id", get(ws_handler).post(start_roll))
        .layer(Extension(server_tx))
        .layer(CookieManagerLayer::new())
        .with_state(Arc::clone(&shared_state));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3030));
    println!("Websocket server {:?}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    if (run_game.await).is_ok() {}
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    server_tx: Extension<GameServerHandle>,
    cookies: Cookies,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let visited = cookies.get(COOKIE_NAME);
    match visited {
        Some(player_id) => {
            let uuid = Uuid::parse_str(player_id.value()).unwrap();
            ws.on_upgrade(move |socket| handle_socket(socket, server_tx, id, uuid, state))
        }
        None => {
            let player_id = Uuid::new_v4();

            let player_clone = player_id;
            cookies.add(Cookie::new(COOKIE_NAME, player_clone.to_string()));
            ws.on_upgrade(move |socket| handle_socket(socket, server_tx, id, player_id, state))
        }
    }
}

async fn start_roll(Path(id): Path<String>, State(state): State<SharedState>, start_roll: String) {
    state.write().unwrap().start_roll.insert(id, start_roll);
    println!("start_rolls - {:?}", state.read().unwrap().start_roll)
}
