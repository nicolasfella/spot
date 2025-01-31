use crate::app::state::{
    browser_state::{BrowserAction, BrowserEvent, BrowserState},
    login_state::{LoginAction, LoginEvent, LoginState},
    playback_state::{PlaybackAction, PlaybackEvent, PlaybackState},
    selection_state::{SelectionAction, SelectionContext, SelectionEvent, SelectionState},
    ScreenName, UpdatableState,
};

#[derive(Clone, Debug)]
pub enum AppAction {
    PlaybackAction(PlaybackAction),
    BrowserAction(BrowserAction),
    SelectionAction(SelectionAction),
    LoginAction(LoginAction),
    Start,
    Raise,
    ShowNotification(String),
    HideNotification,
    ViewNowPlaying,
    // cross-state actions
    QueueSelection,
    DequeueSelection,
    MoveUpSelection,
    MoveDownSelection,
    ChangeSelectionMode(bool),
}

impl AppAction {
    #[allow(non_snake_case)]
    pub fn ViewAlbum(id: String) -> Self {
        BrowserAction::NavigationPush(ScreenName::AlbumDetails(id)).into()
    }

    #[allow(non_snake_case)]
    pub fn ViewArtist(id: String) -> Self {
        BrowserAction::NavigationPush(ScreenName::Artist(id)).into()
    }

    #[allow(non_snake_case)]
    pub fn ViewPlaylist(id: String) -> Self {
        BrowserAction::NavigationPush(ScreenName::PlaylistDetails(id)).into()
    }

    #[allow(non_snake_case)]
    pub fn ViewUser(id: String) -> Self {
        BrowserAction::NavigationPush(ScreenName::User(id)).into()
    }
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    PlaybackEvent(PlaybackEvent),
    BrowserEvent(BrowserEvent),
    SelectionEvent(SelectionEvent),
    LoginEvent(LoginEvent),
    Started,
    Raised,
    NotificationShown(String),
    NotificationHidden,
    NowPlayingShown,
}

pub struct AppState {
    pub playback: PlaybackState,
    pub browser: BrowserState,
    pub selection: SelectionState,
    pub logged_user: LoginState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            playback: Default::default(),
            browser: BrowserState::new(),
            selection: Default::default(),
            logged_user: Default::default(),
        }
    }

    pub fn recommanded_context(&self) -> SelectionContext {
        // TODO: this does not necessarily mean we're actually viewing the playqueue :(
        let is_home = self.browser.current_screen() == &ScreenName::Home;
        if is_home {
            SelectionContext::Queue
        } else {
            SelectionContext::Global
        }
    }

    pub fn update_state(&mut self, message: AppAction) -> Vec<AppEvent> {
        match message {
            AppAction::Start => vec![AppEvent::Started],
            AppAction::ShowNotification(c) => vec![AppEvent::NotificationShown(c)],
            AppAction::HideNotification => vec![AppEvent::NotificationHidden],
            AppAction::ViewNowPlaying => vec![AppEvent::NowPlayingShown],
            AppAction::Raise => vec![AppEvent::Raised],
            AppAction::QueueSelection => {
                for track in self.selection.take_selection() {
                    self.playback.queue(track);
                }
                vec![
                    SelectionEvent::SelectionModeChanged(false).into(),
                    PlaybackEvent::PlaylistChanged.into(),
                ]
            }
            AppAction::DequeueSelection => {
                for track in self.selection.take_selection() {
                    self.playback.dequeue(&track.id);
                }
                vec![
                    SelectionEvent::SelectionModeChanged(false).into(),
                    PlaybackEvent::PlaylistChanged.into(),
                ]
            }
            AppAction::MoveDownSelection => {
                if let Some(song) = self.selection.peek_selection().first() {
                    if self.playback.move_down(&song.id) {
                        return vec![PlaybackEvent::PlaylistChanged.into()];
                    }
                }
                vec![]
            }
            AppAction::MoveUpSelection => {
                if let Some(song) = self.selection.peek_selection().first() {
                    if self.playback.move_up(&song.id) {
                        return vec![PlaybackEvent::PlaylistChanged.into()];
                    }
                }
                vec![]
            }
            AppAction::ChangeSelectionMode(active) => {
                let context = if active {
                    Some(self.recommanded_context())
                } else {
                    None
                };
                if let Some(active) = self.selection.set_mode(context) {
                    vec![SelectionEvent::SelectionModeChanged(active).into()]
                } else {
                    vec![]
                }
            }
            AppAction::PlaybackAction(a) => self
                .playback
                .update_with(a)
                .into_iter()
                .map(AppEvent::PlaybackEvent)
                .collect(),
            AppAction::BrowserAction(a) => self
                .browser
                .update_with(a)
                .into_iter()
                .map(AppEvent::BrowserEvent)
                .collect(),
            AppAction::SelectionAction(a) => self
                .selection
                .update_with(a)
                .into_iter()
                .map(AppEvent::SelectionEvent)
                .collect(),
            AppAction::LoginAction(a) => self.logged_user.update_with(a).into_iter().collect(),
        }
    }
}
