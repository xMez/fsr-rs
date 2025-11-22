use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub thresholds: [i32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub name: String,
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profiles {
    pub profiles: HashMap<String, Profile>,
    pub current_profile: String,
    pub default_profile: String, // New field for default profile
    pub players: HashMap<String, Player>,
    pub current_player: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Command {
    UpdateThreshold {
        profile_name: String,
        threshold_index: usize,
        value: i32,
    },
    AddProfile {
        name: String,
        thresholds: [i32; 4],
    },
    RemoveProfile {
        name: String,
    },
    ChangeProfile {
        name: String,
    },
    ChangePlayer {
        name: String,
    },
    SetDefaultProfile {
        name: String,
    },
    GetCurrentThresholds,
    GetSensorValues, // Kept for backward compatibility
    StartSensorStream,
    StopSensorStream,
    Subscribe {
        event_types: Vec<String>,
    },
    Unsubscribe {
        event_types: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Response {
    pub success: bool,
    pub message: String,
    pub data: Option<Profiles>,
    pub sensor_values: Option<[i32; 4]>,
    pub response_type: Option<String>, // "command_response", "sensor_stream"
}

pub const PROFILES_FILE: &str = "profiles.json";

pub async fn load_profiles() -> Profiles {
    match fs::read_to_string(PROFILES_FILE) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Profiles {
            profiles: HashMap::new(),
            current_profile: String::new(),
            default_profile: String::new(),
            players: HashMap::new(),
            current_player: String::new(),
        }),
        Err(_) => Profiles {
            profiles: HashMap::new(),
            current_profile: String::new(),
            default_profile: String::new(),
            players: HashMap::new(),
            current_player: String::new(),
        },
    }
}

pub async fn save_profiles(
    profiles: &Profiles,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let json = serde_json::to_string_pretty(profiles)?;
    fs::write(PROFILES_FILE, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // --- Test helpers ---
    fn make_profile(thresholds: [i32; 4]) -> Profile {
        Profile { thresholds }
    }

    fn make_profiles_single(
        name: &str,
        thresholds: [i32; 4],
        current: &str,
        default_: &str,
    ) -> Profiles {
        Profiles {
            profiles: HashMap::from([(name.to_string(), Profile { thresholds })]),
            current_profile: current.to_string(),
            default_profile: default_.to_string(),
            players: HashMap::new(),
            current_player: String::new(),
        }
    }

    fn make_profiles_two(
        name1: &str,
        thresholds1: [i32; 4],
        name2: &str,
        thresholds2: [i32; 4],
        current: &str,
        default_: &str,
    ) -> Profiles {
        Profiles {
            profiles: HashMap::from([
                (
                    name1.to_string(),
                    Profile {
                        thresholds: thresholds1,
                    },
                ),
                (
                    name2.to_string(),
                    Profile {
                        thresholds: thresholds2,
                    },
                ),
            ]),
            current_profile: current.to_string(),
            default_profile: default_.to_string(),
            players: HashMap::new(),
            current_player: String::new(),
        }
    }

    fn make_player(name: &str, profile: &str) -> Player {
        Player {
            name: name.to_string(),
            profile: profile.to_string(),
        }
    }

    fn set_players(
        mut profiles: Profiles,
        players: Vec<Player>,
        current_player: Option<&str>,
    ) -> Profiles {
        profiles.players = players
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect::<HashMap<_, _>>();
        if let Some(name) = current_player {
            profiles.current_player = name.to_string();
        }
        profiles
    }

    #[test]
    fn test_profile_serialization() {
        let profile = make_profile([100, 200, 300, 400]);

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: Profile = serde_json::from_str(&json).unwrap();

        assert_eq!(profile, deserialized);
        assert_eq!(profile.thresholds, [100, 200, 300, 400]);
    }

    #[test]
    fn test_profiles_serialization() {
        let profiles = make_profiles_single("Profile1", [10, 20, 30, 40], "Profile1", "");

        let json = serde_json::to_string_pretty(&profiles).unwrap();
        let deserialized: Profiles = serde_json::from_str(&json).unwrap();

        assert_eq!(profiles, deserialized);
        assert_eq!(profiles.profiles.len(), 1);
        assert_eq!(profiles.current_profile, "Profile1");
    }

    #[test]
    fn test_command_serialization() {
        let command = Command::UpdateThreshold {
            profile_name: "Profile1".to_string(),
            threshold_index: 0,
            value: 100,
        };

        let json = serde_json::to_string(&command).unwrap();
        let deserialized: Command = serde_json::from_str(&json).unwrap();

        assert_eq!(command, deserialized);
    }

    #[test]
    fn test_response_serialization() {
        let response = Response {
            success: true,
            message: "Success".to_string(),
            data: Some(make_profiles_single(
                "Profile1",
                [10, 20, 30, 40],
                "Profile1",
                "",
            )),
            sensor_values: None,
            response_type: Some("command_response".to_string()),
        };

        let json = serde_json::to_string_pretty(&response).unwrap();
        let deserialized: Response = serde_json::from_str(&json).unwrap();

        assert_eq!(response, deserialized);
    }

    #[test]
    fn test_profiles_equality() {
        let profiles1 = make_profiles_single("Profile1", [10, 20, 30, 40], "Profile1", "");
        let profiles2 = make_profiles_single("Profile1", [10, 20, 30, 40], "Profile1", "");
        let profiles3 = make_profiles_single("Profile2", [10, 20, 30, 40], "Profile1", "");

        assert_eq!(profiles1, profiles2);
        assert_ne!(profiles1, profiles3);
    }

    #[test]
    fn test_profile_debug() {
        let profile = make_profile([100, 200, 300, 400]);

        let debug_str = format!("{:?}", profile);
        assert!(debug_str.contains("100"));
        assert!(debug_str.contains("200"));
        assert!(debug_str.contains("300"));
        assert!(debug_str.contains("400"));
    }

    #[test]
    fn test_profiles_debug() {
        let profiles = make_profiles_single("Profile1", [10, 20, 30, 40], "Profile1", "");

        let debug_str = format!("{:?}", profiles);
        assert!(debug_str.contains("Profile1"));
        assert!(debug_str.contains("10"));
        assert!(debug_str.contains("20"));
        assert!(debug_str.contains("30"));
        assert!(debug_str.contains("40"));
    }

    #[test]
    fn test_command_debug() {
        let command = Command::UpdateThreshold {
            profile_name: "Profile1".to_string(),
            threshold_index: 0,
            value: 100,
        };

        let debug_str = format!("{:?}", command);
        assert!(debug_str.contains("Profile1"));
        assert!(debug_str.contains("0"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_get_current_thresholds_command() {
        let command = Command::GetCurrentThresholds;
        let json = serde_json::to_string(&command).unwrap();
        let deserialized: Command = serde_json::from_str(&json).unwrap();
        assert_eq!(command, deserialized);
    }

    #[test]
    fn test_response_debug() {
        let response = Response {
            success: true,
            message: "Success".to_string(),
            data: Some(make_profiles_single(
                "Profile1",
                [10, 20, 30, 40],
                "Profile1",
                "",
            )),
            sensor_values: None,
            response_type: Some("command_response".to_string()),
        };

        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("Success"));
        assert!(debug_str.contains("Profile1"));
        assert!(debug_str.contains("10"));
        assert!(debug_str.contains("20"));
        assert!(debug_str.contains("30"));
        assert!(debug_str.contains("40"));
    }

    #[test]
    fn test_profiles_clone() {
        let original = make_profiles_single("Profile1", [10, 20, 30, 40], "Profile1", "");

        let cloned = original.clone();
        assert_eq!(original, cloned);
        assert_eq!(original.profiles.len(), cloned.profiles.len());
        assert_eq!(original.current_profile, cloned.current_profile);
    }

    #[test]
    fn test_profile_clone() {
        let original = make_profile([100, 200, 300, 400]);

        let cloned = original.clone();
        assert_eq!(original, cloned);
        assert_eq!(original.thresholds, cloned.thresholds);
    }

    #[tokio::test]
    async fn test_json_parsing_valid_profile() {
        let json_str = r#"{"thresholds":[100,200,300,400]}"#;
        let parsed: Result<Profile, _> = serde_json::from_str(json_str);

        assert!(parsed.is_ok());
        let profile = parsed.unwrap();
        assert_eq!(profile.thresholds, [100, 200, 300, 400]);
    }

    #[tokio::test]
    async fn test_json_parsing_invalid_profile() {
        let invalid_json = r#"{"thresholds":[100,200,300]}"#; // missing threshold 4
        let parsed: Result<Profile, _> = serde_json::from_str(invalid_json);

        assert!(parsed.is_err());
    }

    #[test]
    fn test_threshold_overflow() {
        let profile = Profile {
            thresholds: [i32::MAX, i32::MAX, i32::MAX, i32::MAX],
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: Profile = serde_json::from_str(&json).unwrap();

        assert_eq!(profile.thresholds, [i32::MAX, i32::MAX, i32::MAX, i32::MAX]);
        assert_eq!(profile, deserialized);
    }

    #[test]
    fn test_empty_strings() {
        let profile = Profile {
            thresholds: [0, 0, 0, 0],
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: Profile = serde_json::from_str(&json).unwrap();

        assert_eq!(profile.thresholds, [0, 0, 0, 0]);
        assert_eq!(profile, deserialized);
    }

    #[test]
    fn test_unicode_characters() {
        let profile = Profile {
            thresholds: [100, 200, 300, 400],
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: Profile = serde_json::from_str(&json).unwrap();

        assert_eq!(profile.thresholds, [100, 200, 300, 400]);
        assert_eq!(profile, deserialized);
    }

    #[test]
    fn test_player_serialization() {
        let player = Player {
            name: "Player1".to_string(),
            profile: "Profile1".to_string(),
        };

        let json = serde_json::to_string(&player).unwrap();
        let deserialized: Player = serde_json::from_str(&json).unwrap();

        assert_eq!(player, deserialized);
        assert_eq!(player.name, "Player1");
        assert_eq!(player.profile, "Profile1");
    }

    #[test]
    fn test_player_debug() {
        let player = Player {
            name: "Player1".to_string(),
            profile: "Profile1".to_string(),
        };

        let debug_str = format!("{:?}", player);
        assert!(debug_str.contains("Player1"));
        assert!(debug_str.contains("Profile1"));
    }

    #[test]
    fn test_player_clone() {
        let original = Player {
            name: "Player1".to_string(),
            profile: "Profile1".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(original, cloned);
        assert_eq!(original.name, cloned.name);
        assert_eq!(original.profile, cloned.profile);
    }

    #[test]
    fn test_change_player_command_serialization() {
        let command = Command::ChangePlayer {
            name: "Player1".to_string(),
        };

        let json = serde_json::to_string(&command).unwrap();
        let deserialized: Command = serde_json::from_str(&json).unwrap();

        assert_eq!(command, deserialized);
    }

    #[test]
    fn test_change_player_command_debug() {
        let command = Command::ChangePlayer {
            name: "Player1".to_string(),
        };

        let debug_str = format!("{:?}", command);
        assert!(debug_str.contains("Player1"));
    }

    #[test]
    fn test_profiles_with_players() {
        let profiles_base = make_profiles_two(
            "Profile1",
            [10, 20, 30, 40],
            "Profile2",
            [50, 60, 70, 80],
            "Profile1",
            "",
        );
        let profiles = set_players(
            profiles_base,
            vec![
                make_player("Player1", "Profile1"),
                make_player("Player2", "Profile2"),
            ],
            Some("Player1"),
        );

        let json = serde_json::to_string_pretty(&profiles).unwrap();
        let deserialized: Profiles = serde_json::from_str(&json).unwrap();

        assert_eq!(profiles, deserialized);
        assert_eq!(profiles.profiles.len(), 2);
        assert_eq!(profiles.players.len(), 2);
        assert_eq!(profiles.current_profile, "Profile1");
        assert_eq!(profiles.current_player, "Player1");
    }
}
