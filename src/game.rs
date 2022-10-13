//use modular::*;
use rand::prelude::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp;
use warp::ws::Message;

const MAX_BOUND: f64 = 100.0;
const MIN_BOUND: f64 = 0.0;
const ORE_DISTANCE_BOUND: f64 = 1.0;
const ORE_ARRAY: [&str; 4] = ["IRON", "SANDSTONE", "DRAGONHIDE", "CRYSTAL"];
static NEXT_ORE: AtomicUsize = AtomicUsize::new(1);

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Position {
    x: f64,
    y: f64,
    z: f64,
}

impl Position {
    fn distance(p1: &Position, p2: &Position) -> f64 {
        let x_dist: f64 = (p2.x.abs() - p1.x.abs()).abs();
        let y_dist: f64 = (p2.y.abs() - p1.y.abs()).abs();
        let z_dist: f64 = (p2.z.abs() - p1.z.abs()).abs();

        let hyp: f64 = (x_dist * x_dist) + (y_dist * y_dist) + (z_dist * z_dist);
        return hyp.sqrt();
    }

    fn random_starting_postion() -> Position {
        let mut rng = rand::thread_rng();
        let starting_position = Position {
            x: rng.gen_range(0.0..100.0),
            y: 1.0,
            z: rng.gen_range(0.0..100.0),
        };

        return starting_position;
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Character {
    pub player_id: usize,
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
type Characters = Arc<RwLock<Vec<Character>>>;
type SerializableCharacters = Vec<Character>;

// define type for list of ore positions
type OreVeins = Arc<RwLock<Vec<Ore>>>;
type SerializableOreVeins = Vec<Ore>;

#[derive(Serialize, Deserialize)]
pub struct Instruction {
    mine_id: usize,
    player_position: Position,
}

#[derive(Serialize, Deserialize)]
pub struct SerializableGame {
    characters: SerializableCharacters,
    ore: SerializableOreVeins,
}

#[derive(Clone)]
pub struct Game {
    pub characters: Characters,
    pub ore: OreVeins,
}

fn mine_ore<'a>(mut character: Character, vein: Ore) -> (Character, bool) {
    if check_proximity(&character.position, &vein.position) {
        let ore_type = &vein.ore_type;
        let ore_inv_amount = vein.amount
            + if character.inventory.contains_key(&vein.ore_type) {
                character.inventory[&vein.ore_type]
            } else {
                0
            };

        character
            .inventory
            .insert(ore_type.to_string(), ore_inv_amount);

        return (
            Character {
                player_id: character.player_id.clone(),
                position: character.position.clone(),
                inventory: character.inventory.clone(),
            },
            true,
        );
    } else {
        println!("Player proximity to ore invalid");
        return (character.clone(), false);
    }
}

fn check_proximity(character_position: &Position, vein_position: &Position) -> bool {
    let dist: f64 = Position::distance(character_position, vein_position);

    if dist < 0.25 {
        return true;
    }

    false
}

pub async fn spawn_character(player_id: usize) -> Character {
    let starting_position = Position::random_starting_postion();
    return Character {
        player_id: player_id,
        position: starting_position,
        inventory: HashMap::default(),
    };
}

pub async fn spawn_ore<'a>(current_ore: &'a Vec<Ore>) -> Ore {
    let mut rng = rand::thread_rng();

    let ore_position: Position = 'outer_loop: loop {
        'middle_loop: loop {
            let first_ore: Ore;

            if current_ore.is_empty() {
                first_ore = Ore {
                    ore_id: NEXT_ORE.fetch_add(1, Ordering::Relaxed),
                    ore_type: ORE_ARRAY.choose(&mut rng).unwrap().to_string(),
                    amount: rng.gen_range(1..3) as usize,
                    position: Position {
                        x: 50.0,
                        y: 1.0,
                        z: 50.0,
                    },
                };

                return first_ore;
            } else {
                first_ore = current_ore.iter().next().unwrap().clone();
            }

            let new_ore_candidate_position = Position::random_starting_postion();

            let new_ore_position_delta =
                Position::distance(&new_ore_candidate_position, &first_ore.position);
            if new_ore_position_delta.clone() > ORE_DISTANCE_BOUND {
                println!("no position conflict");
            } else {
                break 'middle_loop;
            };

            while new_ore_position_delta > ORE_DISTANCE_BOUND {
                for ore in current_ore {
                    let new_ore_position_delta =
                        Position::distance(&new_ore_candidate_position, &ore.position);

                    if new_ore_position_delta > ORE_DISTANCE_BOUND {
                        println!("no position conflict");
                    } else {
                        break 'middle_loop;
                    }

                    break 'outer_loop new_ore_candidate_position;
                }
            }
        }
    };

    let ore_type: &str = *ORE_ARRAY.choose(&mut rng).unwrap();
    let ore_id = NEXT_ORE.fetch_add(1, Ordering::Relaxed);

    return Ore {
        ore_id: ore_id,
        ore_type: ore_type.to_string(),
        amount: rng.gen_range(1..3) as usize,
        position: ore_position,
    };
}

pub async fn initialize_game() -> Game {
    println!("initialize game");

    // definte ore and character position maps
    let characters: Characters = Characters::default();
    let ore_veins: OreVeins = OreVeins::default();

    let game_state: Game = Game {
        characters: characters,
        ore: ore_veins,
    };

    let mut counter: usize = 0;

    while counter < 5 {
        let next_ore: Ore = spawn_ore(&game_state.ore.read().await.clone()).await;
        game_state.ore.write().await.push(next_ore);
        counter += 1;
    }

    // initialize ore position
    return game_state;
}

pub async fn get_game_state(game_state: &Game) -> Message {
    println!("get game state");
    // TODO: serialize and broadcast game state

    let current_ore: SerializableOreVeins = game_state.ore.read().await.clone();
    let current_chars: SerializableCharacters = game_state.characters.read().await.clone();

    let serializable_game: SerializableGame = SerializableGame {
        characters: current_chars,
        ore: current_ore,
    };

    let serialized_game = serde_json::to_string(&serializable_game).unwrap();

    return Message::text(serialized_game);
}

pub async fn execute_game(player_id: usize, msg: Message, game_state: &Game) {
    println!("execute game {:?}", &msg);

    //deserialize message
    let str_message: &str = msg.to_str().unwrap_or_else(|error| {
        panic!("Not a valid game instruction: {:?}", error);
    });
    let game_instruction: Instruction = serde_json::from_str(str_message).unwrap();

    let mut current_ore_one = game_state.ore.read().await.clone();
    let mut current_ore_two = current_ore_one.clone();
    let mut current_chars = game_state.characters.read().await.clone();

    // mining
    let mut character_updated: Character = if current_ore_one
        .iter()
        .any(|e| e.ore_id == game_instruction.mine_id)
    {
        let (character_with_ore, mined) = mine_ore(
            current_chars.remove(
                current_chars
                    .iter()
                    .position(|x| x.player_id == player_id)
                    .expect("player not found"),
            ),
            current_ore_one.remove(
                current_ore_one
                    .iter()
                    .position(|x| x.ore_id == game_instruction.mine_id)
                    .expect("ore not found"),
            ),
        );

        if mined {
            current_ore_two.remove(
                current_ore_two
                    .iter()
                    .position(|x| x.ore_id == game_instruction.mine_id)
                    .expect("ore not found"),
            );

            let new_ore = spawn_ore(&current_ore_two).await;

            // TODO add error handling
            game_state.ore.write().await.remove(
                game_state
                    .ore
                    .read()
                    .await
                    .iter()
                    .position(|x| x.ore_id == game_instruction.mine_id)
                    .expect("ore not found"),
            );

            game_state.ore.write().await.push(new_ore);
        } else {
            println!("mining unsuccessful");
        }
        character_with_ore
    } else {
        println!("not a valid ore id");

        let character: Character = current_chars.remove(
            current_chars
                .iter()
                .position(|x| x.player_id == player_id)
                .expect("player not found"),
        );
        character
    };

    //update character position
    character_updated.position = game_instruction.player_position;

    //remove old character
    game_state.characters.write().await.remove(
        game_state
            .characters
            .read()
            .await
            .iter()
            .position(|x| x.player_id == player_id)
            .expect("player not found"),
    );

    //record new state with replaced character
    game_state
        .characters
        .write()
        .await
        .push(character_updated.clone());
}
