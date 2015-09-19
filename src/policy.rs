use api::IngressMessage;

pub trait Policy {
    fn allow(&self, _msg: &IngressMessage) -> bool;
}

pub const ANONYMOUS: &'static Policy = &AnonymousPolicy;

#[derive(Clone)]
pub struct AnonymousPolicy;

impl Policy for AnonymousPolicy {
    fn allow(&self, msg: &IngressMessage) -> bool {
        match *msg {
            IngressMessage::Register(_) => true,
            IngressMessage::Part => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct DefaultPolicy;

impl Policy for DefaultPolicy {
    fn allow(&self, msg: &IngressMessage) -> bool {
        match *msg {
            IngressMessage::Part => true,
            IngressMessage::Message(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct OwnerPolicy;

impl Policy for OwnerPolicy {
    fn allow(&self, _msg: &IngressMessage) -> bool {
        true
    }
}