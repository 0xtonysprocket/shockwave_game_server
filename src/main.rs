use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use warp::ws::Message;
use warp::Filter;

mod broadcaster;
mod connection;

static NEXT_UUID: AtomicUsize = AtomicUsize::new(1);

pub struct Player {
    pub player_id: usize,
    // pub topics: Vec<String>, for subcribing to topics later on as you move aorund
    pub sender: mpsc::UnboundedSender<Result<Message, warp::Error>>,
}

// define type for dictionary of players
type Players = Arc<RwLock<HashMap<usize, Player>>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // Init hashmap to track active players
    let active_players = Players::default();

    // GET /join_game -> websocket upgrade
    println!("Configuring websocket entry point to join game");
    let join_game = warp::path("join_game")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws, players| {
            ws.on_upgrade(move |socket| connection::player_connection(socket, players, NEXT_UUID))
        });

    println!("Starting Game Broadcast");
    tokio::task::spawn(async move {
        broadcaster::broadcast(&active_players).await;
    });

    let routes = join_game;

    warp::serve(routes).run(([127, 0, 0, 1], 12345)).await;
}
