use futures::channel::mpsc::UnboundedSender;
use librespot::core::spotify_id::SpotifyId;

use crate::app::backend::Command;
use crate::app::components::EventListener;
use crate::app::AppEvent;

pub struct PlayerNotifier {
    sender: UnboundedSender<Command>,
}

impl PlayerNotifier {
    pub fn new(sender: UnboundedSender<Command>) -> Self {
        Self { sender }
    }
}

impl EventListener for PlayerNotifier {
    fn on_event(&mut self, event: &AppEvent) {
        let command = match event {
            AppEvent::FreshTokenRequested => Some(Command::RefreshToken),
            AppEvent::TrackPaused => Some(Command::PlayerPause),
            AppEvent::TrackResumed => Some(Command::PlayerResume),
            AppEvent::TrackChanged(id) => SpotifyId::from_base62(&id).ok().map(Command::PlayerLoad),
            AppEvent::LoginStarted(username, password) => {
                Some(Command::Login(username.to_owned(), password.to_owned()))
            }
            AppEvent::TrackSeeked(position) => Some(Command::PlayerSeek(*position)),
            _ => None,
        };

        if let Some(command) = command {
            self.sender.unbounded_send(command).unwrap_or_else(|_| {
                println!("Could not send message to player");
            });
        }
    }
}
