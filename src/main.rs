#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate websocket;
extern crate hyper;
extern crate serde;
extern crate serde_json;

use std::mem;
use std::thread;
use std::sync::mpsc;
use std::collections::{HashMap, BTreeMap, VecDeque};
use websocket::{Server, Message, Sender, Receiver};
use websocket::header::WebSocketProtocol;
use websocket::stream::WebSocketStream;
use websocket::server::Connection;
use websocket::result::WebSocketResult;

mod api;
use api::IngressMessage;
mod policy;

use policy::Policy;

mod client {
    use websocket::client::Client;
    use websocket::dataframe::DataFrame as DF;
    use websocket::server::receiver::Receiver as R;
    use websocket::server::sender::Sender as S;
    use websocket::stream::WebSocketStream as WSS;

    pub type WsSender = S<WSS>;
    pub type WsReceiver = R<WSS>;
    pub type WsClient = Client<DF, S<WSS>, R<WSS>>;
}
use self::client::{WsSender, WsReceiver};

macro_rules! ret_err(
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(e) => { println!("Line {}: {}", line!(), e); return; }
        }
    }}
);

type WsConn = Connection<WebSocketStream, WebSocketStream>;

enum ChanMessage {
    Introduce(UserId, WsSender),
    Status(String),
    Message(UserId, api::IngressMessage),
}

pub enum User {
    Anonymous,
    Registered(RegisteredUser),
}

impl User {
    fn is_anonymous(&self) -> bool {
        match *self {
            User::Anonymous => true,
            _ => false
        }
    }

    fn nick(&self) -> &str {
        match *self {
            User::Anonymous => "Anonymous",
            User::Registered(ref ru) => &ru.nick,
        }
    }

    fn policy(&self) -> &Policy {
        match *self {
            User::Anonymous => policy::ANONYMOUS,
            User::Registered(ref ru) => &*ru.policy,
        }
    }
}

pub struct RegisteredUser {
    oauth_token: Option<String>,
    nick: String,
    policy: Box<Policy>,
}

impl User {
    pub fn anonymous() -> User {
        User::Anonymous
    }

    pub fn registered(nick: &str) -> User {
        User::Registered(RegisteredUser {
            oauth_token: None,
            nick: nick.to_string(),
            policy: Box::new(policy::DefaultPolicy),
        })
    }
}

#[derive(Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
#[derive(Serialize, Deserialize)]
pub struct UserId(pub u64);

struct Channel {
    // nicks: HashMap<String, UserId>,
    users: HashMap<UserId, User>,
    clients: BTreeMap<UserId, WsSender>,
    dj_queue: VecDeque<UserId>,
    play_queue: Vec<api::PlayItem>,
    rx: mpsc::Receiver<ChanMessage>,
}

impl Channel {
    pub fn new(rx: mpsc::Receiver<ChanMessage>) -> Channel {
        Channel {
            // nicks: HashMap::new(),
            users: HashMap::new(),
            clients: BTreeMap::new(),
            dj_queue: VecDeque::new(),
            play_queue: Vec::new(),
            rx: rx,
        }
    }

    pub fn dispatch_msg(&mut self, msg: api::EgressMessage) {
        let message = serde_json::to_string(&msg).unwrap();
        let mut dead_uid = Vec::new();

        for (uid, client) in self.clients.iter_mut() {
            if let Err(err) = client.send_message(Message::Text(message.clone())) {
                println!("dropping a dead client: {:?}", err);
                dead_uid.push(*uid);
            }
        }

        for uid in dead_uid.into_iter() {
            self.clients.remove(&uid);
        }
    }

    fn handle_register(&mut self, uid: UserId, reg: &api::RegisterMessage) {
        {
            let user = self.users.get_mut(&uid).unwrap();
            if !user.is_anonymous() {
                // FIXME: nick-changing not support ATM
                return;
            }
            mem::replace(user, User::registered(&reg.nick));
        }
        self.dispatch_msg(api::EgressMessage::Join(api::Join {
            when: 0,
            uid: uid,
            nick: reg.nick.clone(),
        }));
    }

    fn handle_dj_queue(&mut self, uid: UserId) {
        if !self.dj_queue.iter().any(|&u| u == uid) {
            self.dj_queue.push_back(uid)
        }
    }

    fn handle_dj_unqueue(&mut self, uid: UserId) {
        let old = mem::replace(&mut self.dj_queue, VecDeque::new());
        self.dj_queue.extend(old.into_iter().filter(|&u| u != uid));
    }

    fn handle_message(&mut self, uid: UserId, msg: &str) {
        self.dispatch_msg(api::EgressMessage::UserMessage(api::UserMessage {
            when: 0,
            uid: uid,
            body: msg.to_string(),
        }));
    }

    pub fn handle_msg(&mut self, uid: UserId, msg: &api::IngressMessage) {
        use api::IngressMessage as IM;
        if let Some(user) = self.users.get(&uid) {
            if !user.policy().allow(msg) {
                println!("user {:?} send disallowed message: {:?}", user.nick(), msg);
                return;
            }
        } else {
            println!("{:?} not found. dropping message", uid);
            return;
        }

        match *msg {
            IM::Register(ref reg) => self.handle_register(uid, reg),
            IM::Disconnect => {
                self.dispatch_msg(api::EgressMessage::Part(api::Part {
                    when: 0,
                    uid: uid,
                }));
            },
            IM::Skip => {
                let pbm = api::PlaybackMessage::Skip(api::Skip { uid: uid });
                self.dispatch_msg(api::EgressMessage::PlaybackMessage(pbm));
            },
            IM::Part => (),
            IM::DjQueue => self.handle_dj_queue(uid),
            IM::DjUnqueue => self.handle_dj_unqueue(uid),
            IM::Message(ref msg) => self.handle_message(uid, msg),
        }
    }

    pub fn run(mut self) {
        loop {
            let message = ret_err!(self.rx.recv());
            match message {
                ChanMessage::Introduce(uid, new_client) => {
                    self.clients.insert(uid, new_client);
                },
                ChanMessage::Status(body) => {
                    self.dispatch_msg(api::EgressMessage::StatusMessage(api::StatusMessage {
                        when: 0,
                        body: body,
                    }))
                },
                ChanMessage::Message(uid, msg) => {
                    self.handle_msg(uid, &msg)
                }
            }
        }
    }
}


fn client_thread(sender: mpsc::Sender<ChanMessage>, user: UserId, mut client_rx: WsReceiver) {
    let client_addr = ret_err!(client_rx.get_mut().peer_addr());

    for msg in client_rx.incoming_messages() {
        match msg {
            Ok(Message::Text(msg)) => {
                let data: IngressMessage = match serde_json::from_str(&msg) {
                    Ok(imsg) => imsg,
                    Err(err) => {
                        println!("client {} sent invalid message (disconnecting): {:?}", client_addr, err);
                        break;
                    }
                };
                let disconnect = if let IngressMessage::Disconnect = data { true } else { false };
                sender.send(ChanMessage::Message(user, data)).unwrap();
                if disconnect {
                    break;
                }
            },
            Ok(Message::Close(_)) => {
                sender.send(ChanMessage::Message(user, IngressMessage::Disconnect)).unwrap();
            },
            Ok(unhandled) => {
                println!("Unhandled: {:?}", unhandled);
            },
            Err(err) => {
                println!("err: {:?}", err);
                break;
            }
        }
    }
}

fn introduce(sender: mpsc::Sender<ChanMessage>, uid: UserId, conn: WsConn) -> WebSocketResult<WsSender> {
    let request = try!(conn.read_request());
    let headers = request.headers.clone();

    try!(request.validate());
    let mut response = request.accept();
    // assert_eq!(response.status, StatusCode::Ok);

    if let Some(&WebSocketProtocol(ref protocols)) = headers.get() {
        if protocols.contains(&("rust-websocket".to_string())) {
            // We have a protocol we want to use
            response.headers.set(WebSocketProtocol(vec!["rust-websocket".to_string()]));
        }
    }

    let mut client = try!(response.send());
    let ip = client.get_mut_sender().get_mut().peer_addr().unwrap();
    println!("Connection from {}", ip);

    let (client_tx, client_rx) = client.split();
    thread::spawn(move || client_thread(sender, uid, client_rx));
    Ok(client_tx)
}


fn start_channel() -> mpsc::Sender<ChanMessage> {
    let (rx, tx) = mpsc::channel();
    thread::spawn(move || Channel::new(tx).run());
    rx
}

fn main() {
    // Start listening for WebSocket connections
    let ws_server = Server::bind("0.0.0.0:2794").unwrap();

    let chan_rx = start_channel();

    let mut user_id = 1;

    for connection in ws_server {
        if let Ok(conn) = connection {
            let new_uid = UserId(user_id);
            user_id += 1;
            match introduce(chan_rx.clone(), new_uid, conn) {
                Ok(client_rx) => {
                    chan_rx.send(ChanMessage::Introduce(new_uid, client_rx)).unwrap();
                },
                Err(err) => {
                    println!("lol error: {:?}", err);
                }
            }
        }
    }
}