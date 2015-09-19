use serde;

mod fields {
    pub mod playback_message {
        pub const ENQUEUE_ITEM: &'static str = "enqueue_item";
        pub const PLAY_ITEM: &'static str = "play_item";
        pub const SKIP: &'static str = "skip";
    }

    pub mod ingress_message {
        pub const REGISTER: &'static str = "register";
        pub const DISCONNECT: &'static str = "disconnect";
        pub const SKIP: &'static str = "skip";
        pub const PART: &'static str = "part";
        pub const DJ_QUEUE: &'static str = "dj_queue";
        pub const DJ_UNQUEUE: &'static str = "dj_unqueue";
        pub const MESSAGE: &'static str = "message";
    }

    pub mod egress_message {
        pub const CLOCK: &'static str = "clock";
        pub const JOIN: &'static str = "join";
        pub const PART: &'static str = "part";
        pub const STATUS_MESSAGE: &'static str = "status_message";
        pub const USER_MESSAGE: &'static str = "user_message";
        pub const PLAYBACK_MESSAGE: &'static str = "playback_message";
    }
}

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
pub struct Clock {
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

#[derive(Debug)]
pub enum PlaybackMessage {
    EnqueueItem(EnqueueItem),
    PlayItem(PlayItem),
    Skip(Skip),
}

impl serde::Serialize for PlaybackMessage {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer,
    {
        use serde::Serialize;
        use self::PlaybackMessage as PM;
        use self::PlaybackMessageField as PMF;
        match *self {
            PM::EnqueueItem(ref body) => (PMF::EnqueueItem, body).serialize(serializer),
            PM::PlayItem(ref body) => (PMF::PlayItem, body).serialize(serializer),
            PM::Skip(ref body) => (PMF::Skip, body).serialize(serializer),
        }
    }
}

impl serde::Deserialize for PlaybackMessage {
    #[inline]
    fn deserialize<D>(deserializer: &mut D) -> Result<PlaybackMessage, D::Error>
        where D: serde::Deserializer,
    {
        deserializer.visit_seq(PlaybackMessageVisitor)
    }
}

struct PlaybackMessageVisitor;

impl serde::de::Visitor for PlaybackMessageVisitor {
    type Value = PlaybackMessage;

    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<PlaybackMessage, V::Error>
        where V: serde::de::SeqVisitor,
    {
        use serde::de::Error;
        use self::PlaybackMessage as PM;
        use self::PlaybackMessageField as PMF;

        let field: Option<PlaybackMessageField> = try!(visitor.visit());
        let field = try!(field.ok_or(V::Error::syntax("expect a string")));
        let result = match field {
            PMF::EnqueueItem => {
                let body = try!(try!(visitor.visit()).ok_or(V::Error::syntax("truncated list")));
                PM::EnqueueItem(body)
            },
            PMF::PlayItem => {
                let body = try!(try!(visitor.visit()).ok_or(V::Error::syntax("truncated list")));
                PM::PlayItem(body)
            },
            PMF::Skip => {
                let body = try!(try!(visitor.visit()).ok_or(V::Error::syntax("truncated list")));
                PM::Skip(body)
            },
        };

        try!(visitor.end());
        Ok(result)
    }
}

#[derive(Copy, Clone)]
enum PlaybackMessageField {
    EnqueueItem,
    PlayItem,
    Skip,
}

impl serde::Serialize for PlaybackMessageField {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer,
    {
        use serde::Serialize;
        self.name().serialize(serializer)
    }
}

impl serde::Deserialize for PlaybackMessageField {
    fn deserialize<D>(deserializer: &mut D) -> Result<PlaybackMessageField, D::Error>
        where D: serde::Deserializer,
    {
        deserializer.visit(PlaybackMessageFieldVisitor)
    }
}

struct PlaybackMessageFieldVisitor;

impl serde::de::Visitor for PlaybackMessageFieldVisitor {
    type Value = PlaybackMessageField;

    fn visit_str<E>(&mut self, value: &str) -> Result<PlaybackMessageField, E>
        where E: serde::de::Error,
    {
        PlaybackMessageField::from_name(value)
            .map_err(|_e| E::syntax("expect a string"))
    }
}


impl PlaybackMessageField {
    pub fn from_name(name: &str) -> Result<Self, ()> {
        use self::fields::playback_message as field;
        use self::PlaybackMessageField as PMF;
        match name {
            field::ENQUEUE_ITEM => Ok(PMF::EnqueueItem),
            field::PLAY_ITEM => Ok(PMF::PlayItem),
            field::SKIP => Ok(PMF::Skip),
            _ => Err(()),
        }
    }

    pub fn name(&self) -> &'static str {
        use self::fields::playback_message as field;
        use self::PlaybackMessageField as PMF;
        match *self {
            PMF::EnqueueItem => field::ENQUEUE_ITEM,
            PMF::PlayItem => field::PLAY_ITEM,
            PMF::Skip => field::SKIP,
        }
    }
}

#[derive(Debug)]
pub enum EgressMessage {
    Clock(Clock),
    Join(Join),
    Part(Part),
    StatusMessage(StatusMessage),
    UserMessage(UserMessage),
    PlaybackMessage(PlaybackMessage),
}

impl serde::Serialize for EgressMessage {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer,
    {
        use serde::Serialize;
        use self::EgressMessage as EM;
        use self::EgressMessageField as EMF;
        match *self {
            EM::Clock(ref body) => (EMF::Clock, body).serialize(serializer),
            EM::Join(ref body) => (EMF::Join, body).serialize(serializer),
            EM::Part(ref body) => (EMF::Part, body).serialize(serializer),
            EM::StatusMessage(ref body) => (EMF::StatusMessage, body).serialize(serializer),
            EM::UserMessage(ref body) => (EMF::UserMessage, body).serialize(serializer),
            EM::PlaybackMessage(ref body) => (EMF::PlaybackMessage, body).serialize(serializer),
        }
    }
}

impl serde::Deserialize for EgressMessage {
    #[inline]
    fn deserialize<D>(deserializer: &mut D) -> Result<EgressMessage, D::Error>
        where D: serde::Deserializer,
    {
        deserializer.visit_seq(EgressMessageVisitor)
    }
}

struct EgressMessageVisitor;

impl serde::de::Visitor for EgressMessageVisitor {
    type Value = EgressMessage;

    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<EgressMessage, V::Error>
        where V: serde::de::SeqVisitor,
    {
        use serde::de::Error;
        use self::EgressMessage as EM;
        use self::EgressMessageField as EMF;

        let field: Option<EgressMessageField> = try!(visitor.visit());
        let field = try!(field.ok_or(V::Error::syntax("expect a string")));
        let result = match field {
            EMF::Clock => {
                let body = try!(try!(visitor.visit()).ok_or(V::Error::syntax("truncated list")));
                EM::Clock(body)
            },
            EMF::Join => {
                let body = try!(try!(visitor.visit()).ok_or(V::Error::syntax("truncated list")));
                EM::Join(body)
            },
            EMF::Part => {
                let body = try!(try!(visitor.visit()).ok_or(V::Error::syntax("truncated list")));
                EM::Part(body)
            },
            EMF::StatusMessage => {
                let body = try!(try!(visitor.visit()).ok_or(V::Error::syntax("truncated list")));
                EM::StatusMessage(body)
            },
            EMF::UserMessage => {
                let body = try!(try!(visitor.visit()).ok_or(V::Error::syntax("truncated list")));
                EM::UserMessage(body)
            },
            EMF::PlaybackMessage => {
                let body = try!(try!(visitor.visit()).ok_or(V::Error::syntax("truncated list")));
                EM::PlaybackMessage(body)
            },
        };

        try!(visitor.end());
        Ok(result)
    }
}

#[derive(Copy, Clone)]
enum EgressMessageField {
    Clock,
    Join,
    Part,
    StatusMessage,
    UserMessage,
    PlaybackMessage,
}

impl EgressMessageField {
    pub fn from_name(name: &str) -> Result<Self, ()> {
        use self::fields::egress_message as field;
        use self::EgressMessageField as EMF;
        match name {
            field::CLOCK => Ok(EMF::Clock),
            field::JOIN => Ok(EMF::Join),
            field::PART => Ok(EMF::Part),
            field::STATUS_MESSAGE => Ok(EMF::StatusMessage),
            field::USER_MESSAGE => Ok(EMF::UserMessage),
            field::PLAYBACK_MESSAGE => Ok(EMF::PlaybackMessage),
            _ => Err(()),
        }
    }

    pub fn name(&self) -> &'static str {
        use self::fields::egress_message as field;
        use self::EgressMessageField as IMF;
        match *self {
            IMF::Clock => field::CLOCK,
            IMF::Join => field::JOIN,
            IMF::Part => field::PART,
            IMF::StatusMessage => field::STATUS_MESSAGE,
            IMF::UserMessage => field::USER_MESSAGE,
            IMF::PlaybackMessage => field::PLAYBACK_MESSAGE,
        }
    }
}

impl serde::Serialize for EgressMessageField {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer,
    {
        use serde::Serialize;
        self.name().serialize(serializer)
    }
}

impl serde::Deserialize for EgressMessageField {
    fn deserialize<D>(deserializer: &mut D) -> Result<EgressMessageField, D::Error>
        where D: serde::Deserializer,
    {
        deserializer.visit(EgressMessageFieldVisitor)
    }
}

struct EgressMessageFieldVisitor;

impl serde::de::Visitor for EgressMessageFieldVisitor {
    type Value = EgressMessageField;

    fn visit_str<E>(&mut self, value: &str) -> Result<EgressMessageField, E>
        where E: serde::de::Error,
    {
        EgressMessageField::from_name(value)
            .map_err(|_e| E::syntax("expect a string"))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterMessage {
    pub nick: String,
}

#[derive(Debug)]
pub enum IngressMessage {
    Register(RegisterMessage),
    Disconnect,
    Skip,
    Part,
    DjQueue,
    DjUnqueue,
    Message(String),
}

impl serde::Serialize for IngressMessage {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer,
    {
        use serde::Serialize;
        use self::IngressMessage as IM;
        use self::IngressMessageField as IMF;
        match *self {
            IM::Register(ref rm) => (IMF::Register, rm).serialize(serializer),
            IM::Disconnect => (IMF::Disconnect,).serialize(serializer),
            IM::Skip => (IMF::Skip,).serialize(serializer),
            IM::Part => (IMF::Part,).serialize(serializer),
            IM::DjQueue => (IMF::DjQueue,).serialize(serializer),
            IM::DjUnqueue => (IMF::DjUnqueue,).serialize(serializer),
            IM::Message(ref body) => (IMF::Message, body).serialize(serializer),
        }
    }
}

impl serde::Deserialize for IngressMessage {
    #[inline]
    fn deserialize<D>(deserializer: &mut D) -> Result<IngressMessage, D::Error>
        where D: serde::Deserializer,
    {
        deserializer.visit_seq(IngressMessageVisitor)
    }
}

struct IngressMessageVisitor;

impl serde::de::Visitor for IngressMessageVisitor {
    type Value = IngressMessage;

    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<IngressMessage, V::Error>
        where V: serde::de::SeqVisitor,
    {
        use serde::de::Error;
        use self::IngressMessage as IM;
        use self::IngressMessageField as IMF;

        let field: Option<IngressMessageField> = try!(visitor.visit());
        let field = try!(field.ok_or(V::Error::syntax("expect a string")));
        let result = match field {
            IMF::Register => {
                let body = try!(try!(visitor.visit()).ok_or(V::Error::syntax("truncated list")));
                IM::Register(body)
            },
            IMF::Disconnect => IM::Disconnect,
            IMF::Skip => IM::Skip,
            IMF::Part => IM::Part,
            IMF::DjQueue => IM::DjQueue,
            IMF::DjUnqueue => IM::DjUnqueue,
            IMF::Message => {
            let body = try!(try!(visitor.visit()).ok_or(V::Error::syntax("truncated list")));
                IM::Message(body)
            },
        };

        try!(visitor.end());
        Ok(result)
    }
}

#[derive(Copy, Clone)]
enum IngressMessageField {
    Register,
    Disconnect,
    Skip,
    Part,
    DjQueue,
    DjUnqueue,
    Message,
}

impl IngressMessageField {
    pub fn from_name(name: &str) -> Result<Self, ()> {
        use self::fields::ingress_message as field;
        use self::IngressMessageField as IMF;
        match name {
            field::REGISTER => Ok(IMF::Register),
            field::DISCONNECT => Ok(IMF::Disconnect),
            field::SKIP => Ok(IMF::Skip),
            field::PART => Ok(IMF::Part),
            field::DJ_QUEUE => Ok(IMF::DjQueue),
            field::DJ_UNQUEUE => Ok(IMF::DjUnqueue),
            field::MESSAGE => Ok(IMF::Message),
            _ => Err(()),
        }
    }

    pub fn name(&self) -> &'static str {
        use self::fields::ingress_message as field;
        use self::IngressMessageField as IMF;
        match *self {
            IMF::Register => field::REGISTER,
            IMF::Disconnect => field::DISCONNECT,
            IMF::Skip => field::SKIP,
            IMF::Part => field::PART,
            IMF::DjQueue => field::DJ_QUEUE,
            IMF::DjUnqueue => field::DJ_UNQUEUE,
            IMF::Message => field::MESSAGE,
        }
    }
}

impl serde::Serialize for IngressMessageField {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer,
    {
        use serde::Serialize;
        self.name().serialize(serializer)
    }
}

impl serde::Deserialize for IngressMessageField {
    fn deserialize<D>(deserializer: &mut D) -> Result<IngressMessageField, D::Error>
        where D: serde::Deserializer,
    {
        deserializer.visit(IngressMessageFieldVisitor)
    }
}

struct IngressMessageFieldVisitor;

impl serde::de::Visitor for IngressMessageFieldVisitor {
    type Value = IngressMessageField;

    fn visit_str<E>(&mut self, value: &str) -> Result<IngressMessageField, E>
        where E: serde::de::Error,
    {
        IngressMessageField::from_name(value)
            .map_err(|_e| E::syntax("expect a string"))
    }
}
