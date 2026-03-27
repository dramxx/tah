use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const APP_NAME: &str = "tah";
const LEGACY_APP_NAME: &str = "msg";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub token: String,
    pub gist_id: String,
    pub identity: String,
    pub peer: String,
}

impl Config {
     fn config_dir() -> PathBuf {
         dirs::config_dir()
             .unwrap_or_else(|| PathBuf::from("."))
             .join(APP_NAME)
     }

     fn legacy_config_dir() -> PathBuf {
         dirs::config_dir()
             .unwrap_or_else(|| PathBuf::from("."))
             .join(LEGACY_APP_NAME)
     }

    pub fn config_path() -> PathBuf {
         Self::config_dir().join("config.toml")
     }

     pub fn existing_config_path() -> Option<PathBuf> {
         if Self::config_path().exists() {
             Some(Self::config_path())
         } else if Self::legacy_config_path().exists() {
             Some(Self::legacy_config_path())
         } else {
             None
         }
     }

     fn legacy_config_path() -> PathBuf {
         Self::legacy_config_dir().join("config.toml")
     }

     pub fn last_read_path() -> PathBuf {
         Self::config_dir().join("last_read")
     }

     fn legacy_last_read_path() -> PathBuf {
         Self::legacy_config_dir().join("last_read")
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.token.is_empty() {
            return Err(ConfigError::Validation("token cannot be empty".into()));
        }
        if self.gist_id.is_empty() {
            return Err(ConfigError::Validation("gist_id cannot be empty".into()));
        }
        if self.identity.is_empty() {
            return Err(ConfigError::Validation("identity cannot be empty".into()));
        }
        if self.peer.is_empty() {
            return Err(ConfigError::Validation("peer cannot be empty".into()));
        }
         if self.identity == self.peer {
             return Err(ConfigError::Validation(
                 "identity and peer must be different".into(),
             ));
         }
        Ok(())
    }

    pub fn load() -> Result<Self, ConfigError> {
         let path = Self::existing_config_path().ok_or(ConfigError::NotInitialized)?;
         let content = fs::read_to_string(&path).map_err(ConfigError::Io)?;
        let config: Config = toml::from_str(&content).map_err(ConfigError::Parse)?;
        config.validate()?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        self.validate()?;
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(ConfigError::Io)?;
        }
        let content = toml::to_string_pretty(self).map_err(ConfigError::Serialize)?;
         write_string_atomic(&path, &content).map_err(ConfigError::Io)?;
        Ok(())
    }

     pub fn load_last_read() -> i64 {
         for path in [Self::last_read_path(), Self::legacy_last_read_path()] {
             if let Ok(content) = fs::read_to_string(path) {
                 if let Ok(timestamp) = content.trim().parse::<i64>() {
                     return timestamp;
                 }
             }
         }

         0
     }

     pub fn save_last_read(timestamp: i64) -> Result<(), ConfigError> {
         let path = Self::last_read_path();
         if let Some(parent) = path.parent() {
             fs::create_dir_all(parent).map_err(ConfigError::Io)?;
         }

         write_string_atomic(&path, &format!("{timestamp}\n")).map_err(ConfigError::Io)
     }

    fn read_line(prompt: &str) -> Result<String, ConfigError> {
        print!("{}", prompt);
        io::stdout().flush().map_err(ConfigError::Io)?;
        let mut line = String::new();
        io::stdin().read_line(&mut line).map_err(ConfigError::Io)?;
        Ok(line.trim().to_string())
    }

    pub fn interactive_init() -> Result<Self, ConfigError> {
        let token = Self::read_line("Enter your GitHub PAT (gist scope): ")?;
        let gist_id = Self::read_line("Enter the Gist ID: ")?;
        let identity = Self::read_line("Enter your username: ")?;
        let peer = Self::read_line("Enter your peer's username: ")?;

        let config = Config {
            token,
            gist_id,
            identity,
            peer,
        };

        config.validate()?;
        Ok(config)
    }
}

 fn write_string_atomic(path: &Path, content: &str) -> io::Result<()> {
     let temp_path = path.with_extension("tmp");

     #[cfg(unix)]
     {
         use std::os::unix::fs::OpenOptionsExt;

         let mut file = fs::OpenOptions::new()
             .create(true)
             .write(true)
             .truncate(true)
             .mode(0o600)
             .open(&temp_path)?;
         file.write_all(content.as_bytes())?;
         file.sync_all()?;
     }

     #[cfg(not(unix))]
     {
         let mut file = fs::File::create(&temp_path)?;
         file.write_all(content.as_bytes())?;
         file.sync_all()?;
     }

     #[cfg(windows)]
     if path.exists() {
         fs::remove_file(path)?;
     }

     fs::rename(temp_path, path)?;
     Ok(())
 }

#[derive(Debug)]
pub enum ConfigError {
    NotInitialized,
    Validation(String),
    Io(io::Error),
    Parse(toml::de::Error),
    Serialize(toml::ser::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NotInitialized => {
                 write!(f, "error: not initialized. Run 'tah --init'")
            }
            ConfigError::Validation(msg) => {
                write!(f, "error: {}", msg)
            }
            ConfigError::Io(e) => write!(f, "error: could not access config: {}", e),
            ConfigError::Parse(e) => write!(f, "error: could not parse config: {}", e),
            ConfigError::Serialize(e) => write!(f, "error: could not save config: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}
