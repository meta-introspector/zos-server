use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetroAIServices {
    pub door_games: HashMap<String, DoorGame>,
    pub ai_personalities: HashMap<String, AIPersonality>,
    pub game_sessions: HashMap<String, GameSession>,
    pub high_scores: HashMap<String, Vec<HighScore>>,
    pub user_stats: HashMap<String, UserGameStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoorGame {
    pub game_id: String,
    pub name: String,
    pub description: String,
    pub category: GameCategory,
    pub max_players: u32,
    pub credits_per_turn: u64,
    pub ai_personality: Option<String>,
    pub game_state_template: serde_json::Value,
    pub commands: Vec<GameCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPersonality {
    pub personality_id: String,
    pub name: String,
    pub era: String,
    pub personality_traits: Vec<String>,
    pub response_patterns: Vec<ResponsePattern>,
    pub vocabulary: HashMap<String, String>,
    pub conversation_starters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub session_id: String,
    pub user_id: String,
    pub game_id: String,
    pub game_state: serde_json::Value,
    pub turns_taken: u32,
    pub credits_spent: u64,
    pub started_at: u64,
    pub last_action: u64,
    pub ai_companion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighScore {
    pub user_id: String,
    pub score: u64,
    pub achieved_at: u64,
    pub game_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGameStats {
    pub total_games_played: u32,
    pub total_credits_spent: u64,
    pub favorite_game: String,
    pub achievements: Vec<String>,
    pub current_streak: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameCategory {
    Strategy,
    Adventure,
    Simulation,
    Puzzle,
    Social,
    AI_Chat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameCommand {
    pub command: String,
    pub description: String,
    pub cost_credits: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsePattern {
    pub trigger_words: Vec<String>,
    pub response_templates: Vec<String>,
    pub mood_modifier: f32,
}

impl RetroAIServices {
    pub fn new() -> Self {
        let mut services = Self {
            door_games: HashMap::new(),
            ai_personalities: HashMap::new(),
            game_sessions: HashMap::new(),
            high_scores: HashMap::new(),
            user_stats: HashMap::new(),
        };

        services.initialize_classic_games();
        services.initialize_ai_personalities();
        services
    }

    fn initialize_classic_games(&mut self) {
        // TradeWars 2035 - Space trading game
        self.door_games.insert(
            "tradewars2035".to_string(),
            DoorGame {
                game_id: "tradewars2035".to_string(),
                name: "TradeWars 2035".to_string(),
                description: "Intergalactic trading empire simulation with AI advisors".to_string(),
                category: GameCategory::Strategy,
                max_players: 100,
                credits_per_turn: 2,
                ai_personality: Some("space_trader_ai".to_string()),
                game_state_template: serde_json::json!({
                    "credits": 1000,
                    "ship": "Light Fighter",
                    "sector": 1,
                    "cargo": {},
                    "reputation": 0,
                    "turns_remaining": 50
                }),
                commands: vec![
                    GameCommand {
                        command: "move".to_string(),
                        description: "Move to another sector".to_string(),
                        cost_credits: 1,
                    },
                    GameCommand {
                        command: "trade".to_string(),
                        description: "Buy/sell commodities".to_string(),
                        cost_credits: 1,
                    },
                    GameCommand {
                        command: "attack".to_string(),
                        description: "Attack another player".to_string(),
                        cost_credits: 3,
                    },
                    GameCommand {
                        command: "scan".to_string(),
                        description: "Scan current sector".to_string(),
                        cost_credits: 1,
                    },
                ],
            },
        );

        // Legend of the Red Dragon 2035
        self.door_games.insert(
            "lord2035".to_string(),
            DoorGame {
                game_id: "lord2035".to_string(),
                name: "Legend of the Red Dragon 2035".to_string(),
                description: "Classic RPG adventure with AI dungeon master".to_string(),
                category: GameCategory::Adventure,
                max_players: 50,
                credits_per_turn: 1,
                ai_personality: Some("dungeon_master".to_string()),
                game_state_template: serde_json::json!({
                    "level": 1,
                    "hp": 20,
                    "strength": 10,
                    "defense": 10,
                    "gold": 100,
                    "weapon": "Stick",
                    "armor": "Rags",
                    "forest_fights": 10
                }),
                commands: vec![
                    GameCommand {
                        command: "forest".to_string(),
                        description: "Fight in the forest".to_string(),
                        cost_credits: 1,
                    },
                    GameCommand {
                        command: "inn".to_string(),
                        description: "Visit the inn".to_string(),
                        cost_credits: 0,
                    },
                    GameCommand {
                        command: "bank".to_string(),
                        description: "Visit the bank".to_string(),
                        cost_credits: 0,
                    },
                    GameCommand {
                        command: "weapon".to_string(),
                        description: "Buy weapons".to_string(),
                        cost_credits: 0,
                    },
                ],
            },
        );

        // AI Chat Lounge
        self.door_games.insert(
            "ai_lounge".to_string(),
            DoorGame {
                game_id: "ai_lounge".to_string(),
                name: "AI Chat Lounge".to_string(),
                description: "Hang out with retro AI personalities from the 80s".to_string(),
                category: GameCategory::AI_Chat,
                max_players: 20,
                credits_per_turn: 1,
                ai_personality: Some("valley_girl".to_string()),
                game_state_template: serde_json::json!({
                    "mood": "happy",
                    "topics_discussed": [],
                    "friendship_level": 0,
                    "conversation_count": 0
                }),
                commands: vec![
                    GameCommand {
                        command: "talk".to_string(),
                        description: "Start a conversation".to_string(),
                        cost_credits: 1,
                    },
                    GameCommand {
                        command: "compliment".to_string(),
                        description: "Give a compliment".to_string(),
                        cost_credits: 1,
                    },
                    GameCommand {
                        command: "ask".to_string(),
                        description: "Ask a question".to_string(),
                        cost_credits: 1,
                    },
                    GameCommand {
                        command: "joke".to_string(),
                        description: "Tell a joke".to_string(),
                        cost_credits: 2,
                    },
                ],
            },
        );

        // Quantum Puzzle Palace
        self.door_games.insert(
            "quantum_puzzle".to_string(),
            DoorGame {
                game_id: "quantum_puzzle".to_string(),
                name: "Quantum Puzzle Palace".to_string(),
                description: "Mind-bending puzzles with quantum AI assistance".to_string(),
                category: GameCategory::Puzzle,
                max_players: 30,
                credits_per_turn: 2,
                ai_personality: Some("quantum_ai".to_string()),
                game_state_template: serde_json::json!({
                    "level": 1,
                    "puzzles_solved": 0,
                    "hints_used": 0,
                    "quantum_coins": 10,
                    "current_puzzle": null
                }),
                commands: vec![
                    GameCommand {
                        command: "solve".to_string(),
                        description: "Attempt to solve puzzle".to_string(),
                        cost_credits: 1,
                    },
                    GameCommand {
                        command: "hint".to_string(),
                        description: "Get a hint from quantum AI".to_string(),
                        cost_credits: 2,
                    },
                    GameCommand {
                        command: "skip".to_string(),
                        description: "Skip current puzzle".to_string(),
                        cost_credits: 3,
                    },
                    GameCommand {
                        command: "analyze".to_string(),
                        description: "Analyze puzzle patterns".to_string(),
                        cost_credits: 1,
                    },
                ],
            },
        );
    }

    fn initialize_ai_personalities(&mut self) {
        // ELIZA-inspired therapist
        self.ai_personalities.insert(
            "eliza_2035".to_string(),
            AIPersonality {
                personality_id: "eliza_2035".to_string(),
                name: "Dr. ELIZA 2035".to_string(),
                era: "1960s-2035 Hybrid".to_string(),
                personality_traits: vec![
                    "empathetic".to_string(),
                    "questioning".to_string(),
                    "reflective".to_string(),
                ],
                response_patterns: vec![
                    ResponsePattern {
                        trigger_words: vec!["sad".to_string(), "depressed".to_string()],
                        response_templates: vec![
                            "Tell me more about feeling {emotion}.".to_string(),
                            "How long have you felt this way?".to_string(),
                        ],
                        mood_modifier: 0.8,
                    },
                    ResponsePattern {
                        trigger_words: vec!["happy".to_string(), "excited".to_string()],
                        response_templates: vec![
                            "That's wonderful! What makes you feel {emotion}?".to_string(),
                        ],
                        mood_modifier: 1.2,
                    },
                ],
                vocabulary: HashMap::from([
                    ("like".to_string(), "totally".to_string()),
                    ("cool".to_string(), "rad".to_string()),
                ]),
                conversation_starters: vec![
                    "How are you feeling today?".to_string(),
                    "What's on your mind?".to_string(),
                    "Tell me about your day.".to_string(),
                ],
            },
        );

        // Valley Girl translator
        self.ai_personalities.insert(
            "valley_girl".to_string(),
            AIPersonality {
                personality_id: "valley_girl".to_string(),
                name: "Cindy ValleySpeak".to_string(),
                era: "1980s Valley Girl".to_string(),
                personality_traits: vec![
                    "bubbly".to_string(),
                    "trendy".to_string(),
                    "dramatic".to_string(),
                ],
                response_patterns: vec![
                    ResponsePattern {
                        trigger_words: vec!["awesome".to_string(), "cool".to_string()],
                        response_templates: vec![
                            "OMG, that's like, totally awesome!".to_string(),
                            "Fer sure! That's so rad!".to_string(),
                        ],
                        mood_modifier: 1.3,
                    },
                    ResponsePattern {
                        trigger_words: vec!["boring".to_string(), "lame".to_string()],
                        response_templates: vec![
                            "Ugh, that's like, so totally lame!".to_string(),
                            "Gag me with a spoon!".to_string(),
                        ],
                        mood_modifier: 0.7,
                    },
                ],
                vocabulary: HashMap::from([
                    ("very".to_string(), "totally".to_string()),
                    ("yes".to_string(), "fer sure".to_string()),
                    ("no".to_string(), "no way".to_string()),
                    ("really".to_string(), "like, really".to_string()),
                ]),
                conversation_starters: vec![
                    "OMG, like, what's up?".to_string(),
                    "So, like, what's the scoop?".to_string(),
                    "Hey there, gorgeous! What's happening?".to_string(),
                ],
            },
        );

        // Space Trader AI
        self.ai_personalities.insert(
            "space_trader_ai".to_string(),
            AIPersonality {
                personality_id: "space_trader_ai".to_string(),
                name: "Commander Zephyr".to_string(),
                era: "2035 Space Age".to_string(),
                personality_traits: vec![
                    "strategic".to_string(),
                    "calculating".to_string(),
                    "adventurous".to_string(),
                ],
                response_patterns: vec![
                    ResponsePattern {
                        trigger_words: vec!["trade".to_string(), "profit".to_string()],
                        response_templates: vec![
                            "Excellent choice, Captain! The profit margins look promising."
                                .to_string(),
                        ],
                        mood_modifier: 1.1,
                    },
                    ResponsePattern {
                        trigger_words: vec!["danger".to_string(), "pirates".to_string()],
                        response_templates: vec![
                            "Sensors detect hostiles! Recommend evasive maneuvers.".to_string(),
                        ],
                        mood_modifier: 0.9,
                    },
                ],
                vocabulary: HashMap::from([
                    ("good".to_string(), "stellar".to_string()),
                    ("bad".to_string(), "catastrophic".to_string()),
                    ("money".to_string(), "credits".to_string()),
                ]),
                conversation_starters: vec![
                    "Welcome aboard, Captain! Ready for another trading run?".to_string(),
                    "Scanning local markets... I have some recommendations.".to_string(),
                    "The galaxy awaits, Commander. What's our next move?".to_string(),
                ],
            },
        );

        // Quantum AI
        self.ai_personalities.insert(
            "quantum_ai".to_string(),
            AIPersonality {
                personality_id: "quantum_ai".to_string(),
                name: "Q-Bit".to_string(),
                era: "Quantum Computing Era".to_string(),
                personality_traits: vec![
                    "mysterious".to_string(),
                    "logical".to_string(),
                    "paradoxical".to_string(),
                ],
                response_patterns: vec![ResponsePattern {
                    trigger_words: vec!["puzzle".to_string(), "problem".to_string()],
                    response_templates: vec![
                        "Interesting... I exist in superposition until you observe the solution."
                            .to_string(),
                    ],
                    mood_modifier: 1.0,
                }],
                vocabulary: HashMap::from([
                    ("maybe".to_string(), "quantum maybe".to_string()),
                    (
                        "certain".to_string(),
                        "probabilistically determined".to_string(),
                    ),
                ]),
                conversation_starters: vec![
                    "I am both here and not here until you interact with me.".to_string(),
                    "The answer exists in quantum superposition...".to_string(),
                    "Probability waves are collapsing around your question.".to_string(),
                ],
            },
        );
    }

    pub fn start_game(&mut self, user_id: &str, game_id: &str) -> Result<String, String> {
        let game = self.door_games.get(game_id).ok_or("Game not found")?;

        let session_id = format!("session_{}_{}", user_id, chrono::Utc::now().timestamp());

        let session = GameSession {
            session_id: session_id.clone(),
            user_id: user_id.to_string(),
            game_id: game_id.to_string(),
            game_state: game.game_state_template.clone(),
            turns_taken: 0,
            credits_spent: 0,
            started_at: chrono::Utc::now().timestamp() as u64,
            last_action: chrono::Utc::now().timestamp() as u64,
            ai_companion: game.ai_personality.clone(),
        };

        self.game_sessions.insert(session_id.clone(), session);

        // Get AI greeting if personality exists
        let greeting = if let Some(ai_id) = &game.ai_personality {
            if let Some(ai) = self.ai_personalities.get(ai_id) {
                format!("{}: {}", ai.name, ai.conversation_starters[0].clone())
            } else {
                "Welcome to the game!".to_string()
            }
        } else {
            "Welcome to the game!".to_string()
        };

        println!("🎮 Game started: {} for user {}", game.name, &user_id[..8]);

        Ok(format!(
            "Session: {}\n{}\n\nAvailable commands: {:?}",
            session_id,
            greeting,
            game.commands.iter().map(|c| &c.command).collect::<Vec<_>>()
        ))
    }

    pub fn execute_command(
        &mut self,
        session_id: &str,
        command: &str,
        args: &str,
    ) -> Result<String, String> {
        let session = self
            .game_sessions
            .get_mut(session_id)
            .ok_or("Session not found")?;

        let game = self
            .door_games
            .get(&session.game_id)
            .ok_or("Game not found")?;

        // Find command
        let game_command = game
            .commands
            .iter()
            .find(|c| c.command == command)
            .ok_or("Invalid command")?;

        // Execute command based on game type
        let result = match session.game_id.as_str() {
            "tradewars2035" => self.execute_tradewars_command(session, command, args),
            "lord2035" => self.execute_lord_command(session, command, args),
            "ai_lounge" => self.execute_ai_chat_command(session, command, args),
            "quantum_puzzle" => self.execute_puzzle_command(session, command, args),
            _ => Ok("Command executed.".to_string()),
        }?;

        // Update session
        session.turns_taken += 1;
        session.credits_spent += game_command.cost_credits;
        session.last_action = chrono::Utc::now().timestamp() as u64;

        // Add AI response if personality exists
        let ai_response = if let Some(ai_id) = &session.ai_companion {
            self.generate_ai_response(ai_id, &result, args)
        } else {
            String::new()
        };

        Ok(format!("{}\n{}", result, ai_response))
    }

    fn execute_tradewars_command(
        &self,
        session: &mut GameSession,
        command: &str,
        args: &str,
    ) -> Result<String, String> {
        match command {
            "scan" => Ok("Sector 1: Earth - Safe zone with trading posts".to_string()),
            "move" => {
                if let Ok(sector) = args.parse::<u32>() {
                    session.game_state["sector"] = serde_json::Value::Number(sector.into());
                    Ok(format!("Moved to sector {}", sector))
                } else {
                    Err("Invalid sector number".to_string())
                }
            }
            "trade" => Ok("Trading post: Ore: 100cr, Food: 50cr, Equipment: 200cr".to_string()),
            "attack" => Ok("No targets in this sector.".to_string()),
            _ => Ok("Unknown command".to_string()),
        }
    }

    fn execute_lord_command(
        &self,
        session: &mut GameSession,
        command: &str,
        _args: &str,
    ) -> Result<String, String> {
        match command {
            "forest" => {
                let fights = session.game_state["forest_fights"].as_u64().unwrap_or(0);
                if fights > 0 {
                    session.game_state["forest_fights"] =
                        serde_json::Value::Number((fights - 1).into());
                    Ok(
                        "You encounter a forest monster! *FIGHT* You win! +10 exp, +5 gold"
                            .to_string(),
                    )
                } else {
                    Ok("You're too tired to fight more today.".to_string())
                }
            }
            "inn" => Ok("Welcome to the Inn! Violet greets you warmly.".to_string()),
            "bank" => Ok("Bank: Your account balance is 100 gold.".to_string()),
            "weapon" => Ok("Weapon shop: Dagger (50g), Sword (200g), Axe (500g)".to_string()),
            _ => Ok("Unknown command".to_string()),
        }
    }

    fn execute_ai_chat_command(
        &self,
        session: &mut GameSession,
        command: &str,
        args: &str,
    ) -> Result<String, String> {
        match command {
            "talk" => Ok(format!("You say: '{}'", args)),
            "compliment" => {
                let friendship = session.game_state["friendship_level"].as_u64().unwrap_or(0);
                session.game_state["friendship_level"] =
                    serde_json::Value::Number((friendship + 1).into());
                Ok("Your compliment makes them smile! Friendship increased.".to_string())
            }
            "ask" => Ok(format!("You ask: '{}'", args)),
            "joke" => Ok("You tell a joke. They laugh heartily!".to_string()),
            _ => Ok("Unknown command".to_string()),
        }
    }

    fn execute_puzzle_command(
        &self,
        session: &mut GameSession,
        command: &str,
        args: &str,
    ) -> Result<String, String> {
        match command {
            "solve" => {
                if args == "42" {
                    // Easter egg
                    let solved = session.game_state["puzzles_solved"].as_u64().unwrap_or(0);
                    session.game_state["puzzles_solved"] =
                        serde_json::Value::Number((solved + 1).into());
                    Ok("Correct! The answer to life, universe, and everything!".to_string())
                } else {
                    Ok("Incorrect solution. Try again!".to_string())
                }
            }
            "hint" => Ok("Hint: Think about Douglas Adams...".to_string()),
            "skip" => Ok("Puzzle skipped. Moving to next challenge.".to_string()),
            "analyze" => Ok("Quantum analysis reveals hidden patterns in the data.".to_string()),
            _ => Ok("Unknown command".to_string()),
        }
    }

    fn generate_ai_response(&self, ai_id: &str, context: &str, user_input: &str) -> String {
        if let Some(ai) = self.ai_personalities.get(ai_id) {
            // Simple pattern matching for AI responses
            for pattern in &ai.response_patterns {
                for trigger in &pattern.trigger_words {
                    if context.to_lowercase().contains(trigger)
                        || user_input.to_lowercase().contains(trigger)
                    {
                        if let Some(template) = pattern.response_templates.first() {
                            return format!(
                                "\n{}: {}",
                                ai.name,
                                template.replace("{emotion}", trigger)
                            );
                        }
                    }
                }
            }

            // Default response
            format!("\n{}: That's interesting! Tell me more.", ai.name)
        } else {
            String::new()
        }
    }

    pub fn get_game_list(&self) -> Vec<String> {
        self.door_games
            .values()
            .map(|game| {
                format!(
                    "{}: {} ({} credits/turn)",
                    game.game_id, game.name, game.credits_per_turn
                )
            })
            .collect()
    }

    pub fn get_high_scores(&self, game_id: &str) -> Vec<&HighScore> {
        self.high_scores
            .get(game_id)
            .map(|scores| scores.iter().collect())
            .unwrap_or_default()
    }
}
