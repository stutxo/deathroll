use axum::{extract::Path, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
// use tower_http::cors::{Any, CorsLayer};
use axum_extra::routing::SpaRouter;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    //let cors = CorsLayer::new().allow_origin(Any);

    //let app = Router::new().route("/:id", get(get_id)).layer(cors);

    // let app = Router::new()
    //     .route("/egg", get(hello))
    //     .merge(SpaRouter::new("/assets", "../dist"))
    //     .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let spa = SpaRouter::new("/assets", "../dist");

    let app = Router::new()
        // `SpaRouter` implements `Into<Router>` so it works with `merge`
        .merge(spa)
        // we can still add other routes
        .route("/api/foo", get(hello));

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

async fn hello() -> impl IntoResponse {
    "hello from server!"
}

#[derive(Serialize, Deserialize, Debug)]
struct Id {
    store_id: String,
}
