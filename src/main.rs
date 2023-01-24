mod metadata;
mod ui;
mod config;

use config::create_example;
use config::get_config;
use druid::AppLauncher;
use druid::WindowDesc;
use druid::WindowHandle;
use druid::WindowLevel;
use metadata::Info;
use metadata::PlayerCommand;
use metadata::ScreenLoc;
use metadata::get_metadata;
use metadata::get_player;
use mpris::PlaybackStatus;
use ui::place_widget;
use ui::ui_builder;
use std::sync::mpsc;
use std::time::Duration;
use std::thread;
use std::env;


fn main() {
    let mut offset = (0., 0.);
    let mut location = ScreenLoc::BottomRight;
    match get_config() {
        Some(conf) => {
            offset = conf.offset;
            location = conf.location;
        },
        None => {},
    }

    let args: Vec<String> = env::args().collect();
    if args.contains(&"--bottom-right".to_string()) {
        location = ScreenLoc::BottomRight
    } else if args.contains(&"--bottom-left".to_string()) {
        location = ScreenLoc::BottomLeft
    } else if args.contains(&"--top-right".to_string()) {
        location = ScreenLoc::TopRight
    }  else if args.contains(&"--top-left".to_string()) {
        location = ScreenLoc::TopLeft
    }  

    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        println!("{}", HELP_MSG);
        return;
    }

    if args.contains(&"--config-example".to_string()) || args.contains(&"-c".to_string()) {
        create_example();
        return;
    }

    let (sender, reciever) = mpsc::channel::<PlayerCommand>();

    let main_window = WindowDesc::new(ui_builder(sender))
        .title("Now Playing")
        .transparent(true)
        .show_titlebar(false)
        .window_size((460.0, 160.0))
        .set_position(place_widget(460., 160., &location, offset.clone()))
        .set_level(WindowLevel::Tooltip(WindowHandle::default()));
    let launcher = AppLauncher::with_window(main_window);
    let event_sink = launcher.get_external_handle();
    let sacrifice = location.clone();
    let sacrifice2 = offset.clone();
    //Connect to player
    let _tracker = thread::spawn(move || {
        loop {
            let player = get_player();
            thread::sleep(Duration::new(1, 0));
            let mut information = get_metadata(&player, &sacrifice, sacrifice2.clone());
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
                    information = metadata::get_metadata(&player, &sacrifice, sacrifice2);
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
        minimize: false,
        location,
        offset
    }).unwrap();
}

const HELP_MSG: &str = "Welcome to Now Playing\n\nConnects to mpris to get what your playing and displays it as a widget on your desktop that is always visible\n\n--bottom-right\tMoves widget to the bottom right corner\n--bottom-left\tMoves widget to the bottom left corner\n--top-right\tMoves widget to the top right corner\n--top-left\tMoves widget to the top left corner\n--help | -h\tDisplays this help message\n--config-example | -c\tCreates an example config file\n\nThe widget placement does not account for any bar or dock, but you can specify your own padding in your `$XDG_CONFIG_HOME/now-playing-rs.yml`,along with your preferred starting corner";
