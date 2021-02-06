use futures::channel::mpsc::{channel, Sender};
use futures::future::{FutureExt, TryFutureExt};
use librespot::core::spotify_id::SpotifyId;
use std::rc::Rc;
use std::thread;
use tokio_core::reactor::Core;

use super::AppAction;
use crate::app::credentials;

mod player;
pub use player::{SpotifyError, SpotifyPlayer, SpotifyPlayerDelegate};

pub mod api;
mod api_models;

pub mod cache;

#[derive(Debug, Clone)]
pub enum Command {
    Login(String, String),
    PlayerLoad(SpotifyId),
    PlayerResume,
    PlayerPause,
    PlayerSeek(u32),
    RefreshToken,
}

struct AppPlayerDelegate {
    sender: Sender<AppAction>,
}

impl AppPlayerDelegate {
    fn new(sender: Sender<AppAction>) -> Self {
        Self { sender }
    }
}

impl SpotifyPlayerDelegate for AppPlayerDelegate {
    fn end_of_track_reached(&self) {
        self.sender.clone().try_send(AppAction::Next).unwrap();
    }

    fn login_successful(&self, credentials: credentials::Credentials) {
        self.sender
            .clone()
            .try_send(AppAction::SetLoginSuccess(credentials))
            .unwrap();
    }

    fn refresh_successful(&self, token: String) {
        self.sender
            .clone()
            .try_send(AppAction::SetRefreshedToken(token))
            .unwrap();
    }

    fn report_error(&self, error: SpotifyError) {
        self.sender
            .clone()
            .try_send(AppAction::ShowNotification(format!("{}", error)))
            .unwrap();
    }

    fn notify_playback_state(&self, position: u32) {
        self.sender
            .clone()
            .try_send(AppAction::SyncSeek(position))
            .unwrap();
    }
}

pub fn start_player_service(appaction_sender: Sender<AppAction>) -> Sender<Command> {
    let (sender, receiver) = channel::<Command>(0);
    thread::spawn(move || {
        let mut core = Core::new().unwrap();
        let delegate = Rc::new(AppPlayerDelegate::new(appaction_sender.clone()));
        core.run(
            SpotifyPlayer::new(delegate)
                .start(core.handle(), receiver)
                .boxed_local()
                .compat(),
        )
        .unwrap_or_else(move |_| {
            appaction_sender
                .clone()
                .try_send(AppAction::ShowNotification(
                    "Player crashed, please restart the application.".to_string(),
                ))
                .unwrap();
        })
    });
    sender
}
