use async_session::{async_trait, MemoryStore, Session, SessionStore};
use axum::{
    extract::{ws::WebSocketUpgrade, FromRef, FromRequestParts, Path},
    http::{self, request::Parts, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Extension, RequestPartsExt, Router, TypedHeader,
};
use axum_extra::routing::SpaRouter;
//use axum_extra::routing::SpaRouter;
use game_server::{GameServer, GameServerHandle};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use uuid::Uuid;

use ws::handle_socket;
mod game_server;
mod ws;

const COOKIE_NAME: &str = "deathroll";

#[tokio::main]
async fn main() {
    let (game_server, server_tx) = GameServer::new();
    let run_game = tokio::spawn(game_server.run());

    let spa = SpaRouter::new("/assets", "../dist");
    let store = MemoryStore::new();
    let app = Router::new()
        .merge(spa)
        .route("/ws/:id", get(ws_handler))
        .with_state(store)
        .layer(Extension(server_tx))
        .layer(CookieManagerLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3030));
    println!("Websocket server {:?}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    match run_game.await {
        Ok(_) => {}
        Err(_) => {}
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    server_tx: Extension<GameServerHandle>,
    cookies: Cookies,
) -> impl IntoResponse {
    let visited = cookies.get(COOKIE_NAME);
    match visited {
        Some(player_id) => {
            println!("{:?}", player_id);
            let uuid = Uuid::parse_str(player_id.value()).unwrap();
            return ws.on_upgrade(move |socket| handle_socket(socket, server_tx, id, uuid));
        }
        None => {
            let player_id = Uuid::new_v4();

            let player_clone = player_id.clone();
            cookies.add(Cookie::new(COOKIE_NAME, player_clone.to_string()));
            return ws.on_upgrade(move |socket| handle_socket(socket, server_tx, id, player_id));
        }
    }

    //println!("{:?}", cookies);
}
