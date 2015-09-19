
#[derive(Debug, Serialize, Deserialize)]
pub struct Join {
    pub when: u64,
    pub uid: super::UserId,
    pub nick: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Part {
    pub when: u64,
    pub uid: super::UserId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Time {
    pub when: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusMessage {
    pub when: u64,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserMessage {
    pub when: u64,
    pub uid: super::UserId,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnqueueItem {
    pub when: u64,
    pub uid: super::UserId,
    pub yid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayItem {
    pub when: u64,
    pub uid: super::UserId,
    pub yid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Skip {
    // who skipped
    pub uid: super::UserId,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PlaybackMessage {
    EnqueueItem(EnqueueItem),
    PlayItem(PlayItem),
    Skip(Skip),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EgressMessage {
    Clock(u64),
    Join(Join),
    Part(Part),
    Time(Time),
    StatusMessage(StatusMessage),
    UserMessage(UserMessage),
    PlaybackMessage(PlaybackMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterMessage {
	pub nick: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IngressMessage {
	Register(RegisterMessage),
	Disconnect,
    Skip,
    Part,
    DjQueue,
    DjUnqueue,
    Message(String),
}