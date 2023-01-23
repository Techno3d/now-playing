mod metadata;
mod ui;

use druid::AppLauncher;
use druid::Screen;
use druid::WindowDesc;
use druid::WindowHandle;
use druid::WindowLevel;
use metadata::Info;
use metadata::PlayerCommand;
use metadata::get_metadata;
use metadata::get_player;
use mpris::PlaybackStatus;
use ui::ui_builder;
use std::sync::mpsc;
use std::time::Duration;
use std::thread;

fn main() {
    let (sender, reciever) = mpsc::channel::<PlayerCommand>();
    let rect = Screen::get_display_rect();

    let main_window = WindowDesc::new(ui_builder(sender))
        .title("Now Playing")
        .transparent(true)
        .show_titlebar(false)
        .window_size((460.0, 160.0))
        .set_position((rect.x1-460., rect.y1-160.0))
        .set_level(WindowLevel::Tooltip(WindowHandle::default()));
    let launcher = AppLauncher::with_window(main_window);
    let event_sink = launcher.get_external_handle();

    //Connect to player
    let _tracker = thread::spawn(move || {
        loop {
            let player = get_player();
            thread::sleep(Duration::new(1, 0));
            let mut information = get_metadata(&player);
            event_sink.add_idle_callback(move |data: &mut Info| {
                *data = information;
            });
            let mut tracker = player.track_progress(500).expect("funny");
            loop {
                let tick = tracker.tick();
                if tick.player_quit {
                    break;
                }
                if tick.progress_changed {
                    information = metadata::get_metadata(&player);
                    event_sink.add_idle_callback(move |data: &mut Info| *data = information);
                }
                let command = match reciever.recv_timeout(Duration::from_millis(5)) {
                    Ok(x) => x,
                    Err(_) => continue,
                };
                match command {
                    PlayerCommand::Pause => {
                        if player.get_playback_status().unwrap_or(PlaybackStatus::Playing) == PlaybackStatus::Paused {
                            player.play().unwrap_or_default()
                        } else {
                            player.pause().unwrap_or_default()
                        }
                    },
                    PlayerCommand::Next => player.next().unwrap_or_default(),
                    PlayerCommand::Prev => player.previous().unwrap_or_default(),
                };
            }
        }
    });


    launcher.launch(metadata::Info {
        title: "".to_string(),
        artists: "".to_string(),
        art_url: "".to_string(),
        album_name: "".to_string(),
        art: None,
        can_next: true,
        can_prev: true,
        can_pause: true,
        is_paused: false,
        min: false
    }).unwrap();
}
