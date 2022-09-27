use crate::Players;
use tokio::{sync::mpsc::UnboundedSender, time::Duration};
mod game;

// loop that broadcasts the
pub async fn broadcast(players: &Players) {
    loop {
        tokio::time::sleep(Duration::from_millis(35)).await;

        let active_player_count = players.read().await.len();
        if active_player_count == 0 {
            println!("No players connected, skip sending data");
            continue;
        }
        println!("{} connected player(s)", active_player_count);

        // get updated game state
        let game_state = game::get_game_state();

        //send game state to every player
        players.read().await.iter().for_each(|(_, player)| {
            player.sender.send(game_state);
        });
    }
}
