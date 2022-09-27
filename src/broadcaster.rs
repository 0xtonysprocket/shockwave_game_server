use crate::Players;
use tokio::time::Duration;
mod game;

// loop that broadcasts the
pub async fn broadcast(players: &Players) {
    loop {
        tokio::time::sleep(Duration::from_millis(35)).await;

        let active_player_count = players.lock().await.len();
        if active_player_count == 0 {
            println!("No players connected, skip sending data");
            continue;
        }
        println!("{} connected player(s)", active_player_count);

        let game_state = game::get_game_state();
        players.read().iter().for_each(|(_, player)| {
            if let Some(sender) = &player.addr {
                sender.send(game_state);
            }
        });
    }
}
