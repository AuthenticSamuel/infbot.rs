use std::{env, error::Error, fmt, num::ParseIntError};

#[derive(Debug, Clone)]
pub struct Config {
    pub discord_bot_token: String,
    pub discord_register_guild: Option<u64>,
}

#[derive(Debug)]
pub enum ConfigError {
    MissingVar(&'static str),
    NotUnicode(&'static str),
    ParseGuildId(ParseIntError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingVar(name) => {
                return write!(f, "missing environment variable: {}", name);
            }
            ConfigError::NotUnicode(name) => {
                return write!(f, "environment variable is not valid UTF-8: {}", name);
            }
            ConfigError::ParseGuildId(e) => {
                return write!(f, "DISCORD_REGISTER_GUILD must be a u64: {}", e);
            }
        }
    }
}

impl Error for ConfigError {}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let discord_bot_token: String;
        match env::var("DISCORD_TOKEN") {
            Ok(v) => {
                discord_bot_token = v;
            }
            Err(env::VarError::NotPresent) => {
                return Err(ConfigError::MissingVar("DISCORD_TOKEN"));
            }
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::NotUnicode("DISCORD_TOKEN"));
            }
        }

        let discord_register_guild: Option<u64>;
        match env::var("DISCORD_REGISTER_GUILD") {
            Ok(s) => {
                let t = s.trim();
                if t.is_empty() {
                    discord_register_guild = None;
                } else {
                    match t.parse::<u64>() {
                        Ok(id) => {
                            discord_register_guild = Some(id);
                        }
                        Err(e) => {
                            return Err(ConfigError::ParseGuildId(e));
                        }
                    }
                }
            }
            Err(env::VarError::NotPresent) => {
                discord_register_guild = None;
            }
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::NotUnicode("DISCORD_REGISTER_GUILD"));
            }
        }

        return Ok(Self {
            discord_bot_token,
            discord_register_guild,
        });
    }
}
