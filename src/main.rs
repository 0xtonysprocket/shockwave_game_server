use game::Game;
use pretty_env_logger;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tokio::task;
use warp;
use warp::ws::Message;
use warp::Filter;

mod broadcaster;
mod connection;
mod game;

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

    println!("initialize game");
    let game_state = game::initialize_game().await;

    // GET /join_game -> websocket upgrade
    println!("Configuring websocket entry point to join game");
    let join_game = warp::path("join_game")
        .and(warp::ws())
        .and(with_active_players(active_players.clone()))
        .and(with_game_state(game_state.clone()))
        .map(|ws: warp::ws::Ws, active_players, game_state| {
            ws.on_upgrade(move |socket| {
                connection::player_connection(socket, active_players, game_state)
            })
        });

    println!("Starting Game Broadcast");
    task::spawn(async move {
        broadcaster::broadcast(&active_players).await;
    });

    let routes = join_game;

    warp::serve(routes).run(([127, 0, 0, 1], 12345)).await;
}

fn with_active_players(
    active_players: Players,
) -> impl Filter<Extract = (Players,), Error = Infallible> + Clone {
    warp::any().map(move || active_players.clone())
}

fn with_game_state(
    game_state: Game,
) -> impl Filter<Extract = (Players,), Error = Infallible> + Clone {
    warp::any().map(move || game_state.clone())
}
