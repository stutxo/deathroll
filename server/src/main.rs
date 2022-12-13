use axum::{extract::Path, routing::get, Json, Router};
use axum_extra::routing::SpaRouter;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let spa = SpaRouter::new("/assets", "../dist");

    let app = Router::new().merge(spa).route("/game/:id", get(get_id));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_id(Path(id): Path<String>) -> Json<Id> {
    let reply_id = Id { store_id: id };
    println!("{:?}", reply_id);
    Json(reply_id)
}

#[derive(Serialize, Deserialize, Debug)]
struct Id {
    store_id: String,
}
