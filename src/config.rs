use serde::Deserialize;
use serde::Serialize;
use std::fs;
use directories::BaseDirs;

use crate::metadata::ScreenLoc;

#[derive(Deserialize, Serialize)]
struct Conf {
    top: bool,
    left: bool,
    #[serde(rename="right-pad")]
    right_pad: f64,
    #[serde(rename="top-pad")]
    top_pad: f64
}

impl Conf {
    fn convert_to_config(&self) -> Config {
        let mut location = ScreenLoc::BottomRight;
        if self.top == true && self.left == true {
            location = ScreenLoc::TopLeft;
        } else if self.top == true && self.left == false {
            location = ScreenLoc::TopRight;
        } else if self.top == false && self.left == true {
            location = ScreenLoc::BottomLeft;
        } else if self.top == false && self.left == false {
            location = ScreenLoc::BottomRight;
        }
        return Config { location, offset: (self.right_pad, self.top_pad) }
    }
}

pub struct Config {
    pub location: ScreenLoc,
    pub offset: (f64, f64)
}

pub fn get_config() -> Option<Config> {
    let base_dir = match BaseDirs::new() {
        Some(a) => a,
        None => return None,
    };
    let config_path = base_dir.config_dir().to_str().unwrap().to_string() + "/now-playing-rs.yml";
    let config_file = match fs::read_to_string(config_path) {
        Ok(a) => a,
        Err(_) => return None,
    };

    let config: Conf = match serde_yaml::from_str(&config_file) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Your config file is broken {e}");
            return None
        },
    };
    
    return Some(config.convert_to_config())
    
}
