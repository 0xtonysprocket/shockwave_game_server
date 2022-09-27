use std::sync::atomic::AtomicUsize;
use warp::ws::WebSocket;

use crate::Players;
use tokio::sync::mpsc::UnboundedSender;

mod game;

pub async fn player_connection(ws: WebSocket, active_players: Players, NEXT_UUID: AtomicUsize) {
    // increment id
    let next_id = NEXT_UUID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new player joined: {}", next_id);

    let (player_ws_sender, player_ws_receiver) = websocket_buffer(ws).await;

    // Add player to players list
    active_players.write().await.insert(
        next_id,
        Player {
            player_id: next_id,
            addr: player_ws_sender,
        },
    );

    execute_player_actions(player_ws_receiver, player_id).await;

    // execute_player_actions stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    player_disconnected(player_id, &players).await
}

async fn execute_player_actions(player_ws_receiver: UnboundedSender, player_id: usize) {
    // Every time the user sends a message,
    // execute changes to game state
    while let Some(result) = player_ws_receiver.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", player_id, e);
                break;
            }
        };
        game::execute_game(player_id, msg).await;
    }
}

async fn websocket_buffer(ws: WebSocket) {
    // Split the socket into a sender and receive of messages.
    let (player_ws_sender, mut player_ws_receiver) = ws.split();

    let (player_sender, player_receiver) = mpsc::unbounded_channel();

    // Buffer
    let player_receiver = UnboundedReceiverStream::new(player_receiver);

    tokio::task::spawn(player_receiver.forward(player_ws_sender).map(|result| {
        if let Err(e) = result {
            println!("error sending websocket msg: {}", e);
        }
    }));

    return (player_ws_sender, player_ws_receiver);
}

async fn player_disconnected(id: usize, active_players: &Players) {
    eprintln!("player disconnected: {}", id);

    // Stream closed up, so remove from the player list
    active_players.write().await.remove(&id);
}
