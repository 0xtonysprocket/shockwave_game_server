use crate::game::{get_game_state, Game};
use crate::Players;
use tokio;
use tokio::time::Duration;

// loop that broadcasts the
pub async fn broadcast(players: &Players, game_state: &Game) {
    loop {
        tokio::time::sleep(Duration::from_millis(10000)).await;

        let active_player_count = players.read().await.len();
        if active_player_count == 0 {
            println!("No players connected, skip sending data");
            continue;
        }
        println!("{} connected player(s)", active_player_count);

        // get updated game state
        let game_state_msg = get_game_state(game_state).await;

        //send game state to every player
        players.read().await.iter().for_each(|(_, player)| {
            player
                .sender
                .send(Ok(game_state_msg.clone()))
                .unwrap_or_else(|error| {
                    panic!("Error sending WS message: {:?}", error);
                });
        });
    }
}
