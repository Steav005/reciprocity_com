use serde::{Deserialize, Serialize};
use serde_diff::{SerdeDiff, Diff, Apply};
use oauth2::{AuthorizationCode, RefreshToken};
use std::time::Duration;
use url::Url;
pub use oauth2;
use std::fmt::{Display, Formatter};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Message {
    ClientRequest(ClientRequest),
    Auth(AuthMessage),
    PlayerState(Option<State>),
    UserVoiceState(VoiceState),
    Unexpected(Unexpected)
}

pub enum PatchError{
    DecodeError(rmp_serde::decode::Error),
    WrongVariant()
}

impl Message{
    pub fn patch_player_state(&self, state: &mut PlayerState) -> Result<(), PatchError>{
        if let Message::PlayerState(Some(State::UpdateState(patch))) = self{
            let mut de = rmp_serde::Deserializer::new(patch.as_slice());
            return Apply::apply(&mut de, state).map_err(PatchError::DecodeError);
        }

        Err(PatchError::WrongVariant())
    }

    pub fn generate_patch(old: &PlayerState, new: &PlayerState) -> Result<Vec<u8>, rmp_serde::encode::Error>{
        rmp_serde::to_vec(&Diff::serializable(old, new))
    }

    pub fn generate(&self) -> Result<Vec<u8>, rmp_serde::encode::Error>{
        rmp_serde::to_vec(self)
    }

    pub fn parse(bin: &[u8]) -> Result<Self, rmp_serde::decode::Error>{
        rmp_serde::from_read(bin)
    }
}

impl Display for Message{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::ClientRequest(_) => writeln!(f, "ClientRequest"),
            Message::Auth(_) => writeln!(f, "Auth"),
            Message::PlayerState(_) => writeln!(f, "PlayerState"),
            Message::Unexpected(_) => writeln!(f, "Unexpected"),
            Message::UserVoiceState(_) => writeln!(f, "UserVoiceState"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Unexpected{
    WsMessageTypeString(String),
    ParseError(Vec<u8>, String),
    MessageType(String),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum AuthMessage{
    AuthStatus(bool),
    AuthSuccess(User),
    AuthError(),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum State{
    FullState(Box<PlayerState>),
    UpdateState(Vec<u8>)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ClientRequest {
    Authenticate(Auth),
    AuthStatus(),
    FullPlayerState(),
    VoiceState(),
    Control(PlayerControl),
    End()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Auth{
    Code(AuthorizationCode),
    Token(RefreshToken),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum PlayerControl{
    Resume(),
    Pause(),
    Skip(usize),
    BackSkip(usize),
    SetTime(Duration),
    PlayMode(PlayMode),
    Enqueue(Url),
    Leave(),
    Join(),
}


#[derive(Debug, Deserialize, Serialize, SerdeDiff, Clone, PartialEq)]
pub enum PlayMode{
    Normal,
    LoopAll,
    LoopOne,
}

#[derive(Debug, Deserialize, Serialize, SerdeDiff, Clone, PartialEq)]
pub struct PlayerState{
    pub bot: BotInfo,
    pub paused: bool,
    pub mode: PlayMode,
    pub current: Option<Track>,
    pub history: Vec<Track>,
    pub queue: Vec<Track>,
}

#[derive(Debug, Deserialize, Serialize, SerdeDiff, Clone, PartialEq)]
pub struct BotInfo{
    pub name: String,
    pub avatar: String,
}

#[derive(Debug, Deserialize, Serialize, SerdeDiff, Clone, PartialEq)]
pub struct Track{
    pub len: Duration,
    pub pos: Duration,
    pub title: String,
    pub uri: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct User{
    pub username: String,
    pub id: String,
    pub avatar: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct VoiceState{
    pub channel_id: u64,
    pub channel_name: String,
    pub bot_in_channel: bool,
}