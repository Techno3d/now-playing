use std::sync::Arc;

use mpris::{Player, PlayerFinder, PlaybackStatus};
use druid::{Data, Lens, ImageBuf};

#[derive(Data, Clone, Lens)]
pub struct Info {
    pub title: String,
    pub artists: String,
    pub art_url: String,
    pub album_name: String,
    pub art: Option<Arc<ImageBuf>>,
    pub can_prev: bool,
    pub can_next: bool,
    pub can_pause: bool,
    pub is_paused: bool,
    #[data(ignore)]
    pub minimize: bool,
    #[data(ignore)]
    pub location: ScreenLoc,
    #[data(ignore)]
    pub offset: (f64, f64)
}

pub enum PlayerCommand {
    Pause, Next, Prev
}

pub fn get_metadata(player: &Player, location: &ScreenLoc, offset: (f64, f64)) -> Info {
        let meta = player.get_metadata().unwrap();
        let mut artists: String = String::new();
        for artist in meta.artists().unwrap_or_default() {
            artists += artist;
        }
        //let resp = attohttpc::get(meta.art_url().unwrap_or_default()).send().unwrap();
        let url = meta.art_url().unwrap_or_default();
        let mut buffer: ImageBuf = ImageBuf::default();
        if url != "" {
            let resp = attohttpc::get(url).send().unwrap();
            if resp.is_success() {
                let bytes = resp.bytes().unwrap();
                buffer = ImageBuf::from_data(&bytes).unwrap_or_default();
            }
            
        }
        return Info {
            title: meta.title().unwrap_or_default().to_string(),
            artists,
            art_url: meta.art_url().unwrap_or_default().to_string(),
            album_name: meta.album_name().unwrap_or_default().to_string(),
            art: Some(Arc::new(buffer)),
            can_prev: !player.can_go_previous().unwrap_or_default(),
            can_pause: !player.can_pause().unwrap_or_default(),
            can_next: !player.can_go_next().unwrap_or_default(),
            is_paused: player.get_playback_status().unwrap_or(PlaybackStatus::Playing) == PlaybackStatus::Paused,
            minimize: false,
            location: location.clone(),
            offset
        };
}

pub fn get_player() -> Player {
    let player = PlayerFinder::new().expect("Failed to connect to D-Bus");
    loop {
        let player = match player.find_active() {
            Ok(player) => player,
            Err(_) => continue,
        };
        return player;
    }
}

#[derive(Clone)]
pub enum ScreenLoc {
    TopRight, TopLeft, BottomLeft, BottomRight 
}
