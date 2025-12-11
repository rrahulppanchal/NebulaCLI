use std::{env, fs, io::Write, path::PathBuf};

pub struct Config {
    pub username: String,
}

impl Config {
    pub fn load() -> Self {
        let username = Self::load_username();
        Config { username }
    }

    fn load_username() -> String {
        let config_path = Self::get_config_path();

        if let Ok(content) = fs::read_to_string(&config_path) {
            let username = content.trim().to_string();
            if !username.is_empty() {
                return username;
            }
        }

        "user".to_string()
    }

    pub fn save_username(username: &str) -> std::io::Result<()> {
        let config_path = Self::get_config_path();

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(config_path, username)?;
        Ok(())
    }

    fn get_config_path() -> PathBuf {
        let mut path = env::current_exe().unwrap_or_default();
        path.pop();
        path.push("terminal_config.txt");
        path
    }

    pub fn prompt_for_username() -> Option<String> {
        print!("Enter your username for the terminal (default: user): ");
        std::io::stdout().flush().ok()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok()?;

        let username = input.trim();
        if username.is_empty() {
            Some("user".to_string())
        } else {
            Some(username.to_string())
        }
    }
}
