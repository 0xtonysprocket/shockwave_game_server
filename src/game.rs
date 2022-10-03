//use modular::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::string::String;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp;
use warp::ws::Message;

const MAX_BOUND: f64 = 100.0;
const MIN_BOUND: f64 = 0.0;

#[derive(Serialize, Deserialize)]
pub struct Position {
    x_coordinate: f64,
    y_coordinate: f64,
}

// impl Positon {
//    fn initialize() -> Position {
// here generate a random position between 0 and 100 for each coord that doesn't collide with existing ore or players
//    }

//    fn update_position(x_update: f64, y_update: f64) -> Position {
//        let new_x = modulo!(x_coordinate + x_update, MAX_BOUND);
//        let new_y = modulo!(y_coordinate + y_update, MAX_BOUND);

//       return Position {
//            x_coordinate: new_x,
//            y_coordinate: new_y,
//        };
//    }
//}

#[derive(Serialize, Deserialize)]
pub struct Character {
    player_id: usize,
    position: Position,
    inventory: HashMap<String, usize>,
}

#[derive(Serialize, Deserialize)]
pub struct Ore {
    ore_id: usize,
    ore_type: String,
    amount: usize,
    position: Position,
}

// define type for dictionary of players
type Characters = Arc<RwLock<HashMap<usize, Character>>>;
type SerializableCharacters = HashMap<usize, Character>;

// define type for list of ore positions
type OreVeins = Arc<RwLock<HashMap<usize, Ore>>>;
type SerializableOreVeins = HashMap<usize, Ore>;

#[derive(Serialize, Deserialize)]
pub struct Instruction {
    mine_id: usize,
    player_postion: Position,
}

#[derive(Serialize, Deserialize)]
pub struct SerializableGame {
    characters: SerializableCharacters,
    ore: SerializableOreVeins,
}

pub struct Game {
    characters: Characters,
    ore: OreVeins,
}

fn mine_ore(mut character: Character, vein: Ore) {
    if check_proximity(character.position, vein.position) {
        character.inventory.insert(
            vein.ore_type.clone(),
            vein.amount
                + if character.inventory.contains_key(&vein.ore_type) {
                    character.inventory[&vein.ore_type]
                } else {
                    0
                },
        );
    } else {
        println!("Player proximity to ore invalid")
    }
}

fn check_proximity(character_position: Position, vein_position: Position) -> bool {
    let delta_x: f64 = character_position.x_coordinate - vein_position.x_coordinate;
    let delta_y: f64 = character_position.y_coordinate - vein_position.y_coordinate;

    if delta_x < 0.25 && delta_y < 0.25 {
        return true;
    }

    false
}

pub async fn initialize_game() -> Game {
    println!("initialize game");

    // definte ore and character position maps
    let characters: Characters = Characters::default();
    let ore_veins: OreVeins = OreVeins::default();

    // initialize ore position

    return Game {
        characters: characters,
        ore: ore_veins,
    };
}

pub async fn get_game_state() -> Message {
    println!("get game state");
    //query player character positions
    //query ore positions

    return Message::text("hello");
}

pub async fn execute_game(player_id: usize, msg: Message) {
    println!("execute game {:?}", msg);

    //deserialize message
    let str_message: &str = msg.to_str().unwrap();
    let game_instruction: Instruction = serde_json::from_str(str_message).unwrap();
    // update character position
    // execute mining command if present
    // record new state
}
