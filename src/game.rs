
use std::cmp::Ordering;
use modular::*;

use crate::{MAX_BOUND, MIN_BOUND};

pub struct Player {
    pub player_id: usize,
    // pub topics: Vec<String>, for subcribing to topics later on as you move aorund
    pub sender: mpsc::UnboundedSender<Result<Message, warp::Error>>,
}

pub struct Position {
    x_coordinate: f64,
    y_coordinate: f64,
}

impl Positon {
    fn initialize() -> Position {
        // here generate a random position between 0 and 100 for each coord that doesn't collide with existing ore or players
    }

    fn update_position(x_update: f64, y_update: f64) -> Position {
        let new_x = modulo!(x_coordinate + x_update, MAX_BOUND); 
        let new_y = modulo!(y_coordinate + y_update, MAX_BOUND);

        return Position {x_coordinate = new_x, y_coordinate = new_y}
    }
}

pub struct PlayerCharacter {
    player_id : usize,
    position : Position,
    inventory : HashMap<String>
}

pub struct Ore {
    ore_id : usize,
    ore_type : String,
    position : Position
}

// define type for dictionary of players
type Players = Arc<RwLock<HashMap<usize, Player>>>;

// define type for list of ore positions
type OreVeins = Arc<RwLock<HashMap<usize, Ore>>>;

pub async fn initialize_game() {
    // initialize ore position
}

pub async fn get_game_state() {
    //query player character positions
    //query ore positions
}

pub async fn execute_game() {
    // update character position
    // execute mining command if present
    // record new state
}