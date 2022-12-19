use axum::{
    extract::{ws::WebSocketUpgrade, Path},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use axum_extra::routing::SpaRouter;
use game_server::{GameServer, GameServerHandle};
use std::net::SocketAddr;
use uuid::Uuid;
use ws::handle_socket;
mod game_server;
mod ws;

pub type PlayerId = Uuid;
pub type GameId = String;
pub type Msg = String;

#[tokio::main]
async fn main() {
    let (game_server, server_tx) = GameServer::new();
    let run_game = tokio::spawn(game_server.run());

    let spa = SpaRouter::new("/assets", "../dist");

    let app = Router::new()
        .merge(spa)
        .route("/ws/:id", get(ws_handler))
        .layer(Extension(server_tx));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
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
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, server_tx, id))
}
