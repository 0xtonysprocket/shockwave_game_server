//use modular::*;
use rand::Rng;
use rand::prelude::v
use serde::{Deserialize, Serialize};
use std::array;
use std::collections::HashMap;
use std::hash::Hash;
use std::string::String;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp;
use warp::ws::Message;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crate::game;

const MAX_BOUND: f64 = 100.0;
const MIN_BOUND: f64 = 0.0;
const ORE_DISTANCE_BOUND: f64 = 1.0;
const ORE_ARRAY: Vec<&str>= vec!["IRON", "SANDSTONE", "DRAGONHIDE", "CRYSTAL"];
static NEXT_ORE: AtomicUsize = AtomicUsize::new(1);

#[derive(Serialize, Deserialize, Clone)]
pub struct Position {
    x_coordinate: f64,
    y_coordinate: f64,
}

impl Position {
    fn distance(p1: &Position, p2: &Position) -> f64 {
        let x_dist: f64 = (p2.x_coordinate.abs() - p1.x_coordinate.abs()).abs();
        let y_dist: f64 = (p2.y_coordinate.abs() - p1.y_coordinate.abs()).abs();

        let hyp: f64 = (x_dist * x_dist) + (y_dist * y_dist);
        return hyp.sqrt();
    }
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Character {
    player_id: usize,
    position: Position,
    inventory: HashMap<String, usize>,
}

#[derive(Serialize, Deserialize, Clone)]
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

fn mine_ore(character: &Character, vein: &Ore) -> (Character, bool) {
    if check_proximity(&character.position, &vein.position) {
        let ore_type = vein.ore_type;
        let ore_inv_amount = vein.amount
            + if character.inventory.contains_key(&vein.ore_type) {
                character.inventory[&vein.ore_type]
            } else {
                0
            };

        character.inventory.insert(ore_type, ore_inv_amount);

        return (
            Character {
                player_id: character.player_id,
                position: character.position,
                inventory: character.inventory,
            },
            true,
        );
    } else {
        println!("Player proximity to ore invalid");
        return (*character, false);
    }
}

fn check_proximity(character_position: &Position, vein_position: &Position) -> bool {
    let delta_x: f64 = character_position.x_coordinate - vein_position.x_coordinate;
    let delta_y: f64 = character_position.y_coordinate - vein_position.y_coordinate;

    if delta_x < 0.25 && delta_y < 0.25 {
        return true;
    }

    false
}

pub async fn spawn_ore(current_ore: HashMap<usize, Ore>) -> (usize, Ore) {
    let mut rng = rand::thread_rng();
    
    let ore_position: Position = 'outer_loop: loop {
        'middle_loop: loop {
            let mut new_ore_position_delta: f64;
            let mut first_ore: Ore;

            if let Some(&x) = current_ore.values().next() {
                first_ore = x;
            } else {
                first_ore = Ore {
                    ore_id: NEXT_ORE.fetch_add(1, Ordering::Relaxed), 
                    ore_type: ORE_ARRAY.choose(rng), 
                    amount: rng.gen_range(1..3) as usize, 
                    position: Position{ x_coordinate: 50, y_coordinate: 50}}

                return (first_ore.ore_id, first_ore)
            };

            let new_ore_candidate_position = Position {
                x_coordinate: rng.gen_range(0.0..100.0),
                y_coordinate: rng.gen_range(0.0..100.0),
            };

            let new_ore_position_delta =
                Position::distance(&new_ore_candidate_position, &first_ore.position);
            if new_ore_position_delta.clone() < ORE_DISTANCE_BOUND {
                continue 'middle_loop;
            } else {
                break 'middle_loop;
            };

            'inner_loop: while new_ore_position_delta < ORE_DISTANCE_BOUND {
                for (key, value) in &current_ore {
                    let new_ore_position_delta =
                        Position::distance(&new_ore_candidate_position, &value.position);

                    if new_ore_position_delta < ORE_DISTANCE_BOUND {
                        continue 'inner_loop;
                    } else {
                        break 'middle_loop;
                    }

                    break 'outer_loop new_ore_candidate_position;
                };
            };
        };
    };

    let ore_type: String = ORE_ARRAY.choose(rng);



    return (1, Ore {
        ore_id: NEXT_ORE.fetch_add(1, Ordering::Relaxed), 
        ore_type: ore_type, 
        amount: rng.gen_range(1..3) as usize, 
        position: ore_position})
}

pub async fn initialize_game() -> Game {
    println!("initialize game");

    // definte ore and character position maps
    let characters: Characters = Characters::default();
    let ore_veins: OreVeins = OreVeins::default();

    let counter: usize = 0;

    while counter < 5 {
        let next_ore: Ore = spawn_ore(ore_veins);
        ore_veins.write().await.insert(next_ore.ore_id, next_ore);
        counter += 1;
    }

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

pub async fn execute_game(player_id: usize, msg: Message, game_state: Game) {
    println!("execute game {:?}", msg);

    //deserialize message
    let str_message: &str = msg.to_str().unwrap();
    let game_instruction: Instruction = serde_json::from_str(str_message).unwrap();

    let mut current_ore = game_state.ore.read().await.clone();
    let current_chars = game_state.characters.read().await;

    // mining
    let mut character_updated: Character;

    if current_ore.contains_key(&game_instruction.mine_id) {
        let (character_with_ore, mined) = mine_ore(
            &current_chars[&player_id],
            &current_ore[&game_instruction.mine_id],
        );

        if mined {
            current_ore.remove(&game_instruction.mine_id);

            let (new_ore_id, new_ore) = spawn_ore(current_ore).await;

            // TODO add error handling
            game_state
                .ore
                .write()
                .await
                .remove(&game_instruction.mine_id);

            game_state.ore.write().await.insert(new_ore_id, new_ore);
        } else {
            println!("mining unsuccessful");
        }

        character_updated = character_with_ore;
    } else {
        println!("not a valid ore id");
        character_updated = Character {
            player_id: current_chars[&player_id].player_id,
            position: Position {
                x_coordinate: current_chars[&player_id].position.x_coordinate,
                y_coordinate: current_chars[&player_id].position.y_coordinate,
            },
            inventory: current_chars[&player_id].inventory.clone(),
        };
    };

    //update character position
    character_updated.position = game_instruction.player_postion;

    //record new state
    game_state
        .characters
        .write()
        .await
        .insert(player_id, character_updated);
}
