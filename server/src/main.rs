use axum::{extract::Path, response::Redirect, routing::get, Json, Router};
use axum_extra::routing::SpaRouter;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let spa = SpaRouter::new("/assets", "../dist");

    let app = Router::new().merge(spa).route(
        "/new/:id",
        get(|Path(id)| async {
            // let id = nanoid!(8);
            // let slash = "/".to_owned();
            // let url = slash + &id;
            let url: String = id;
            println!("{:?}", url);
        }),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// fn get_id(Path(id): Path<String>) {
//     let url: String = id;
//     println!("{:?}", url);
// }

#[derive(Serialize, Deserialize, Debug)]
struct Id {
    store_id: String,
}
