use crate::{codec::Encodable, objects::identity::IdentityObject};

#[derive(Debug, Default)]
pub struct RegisterSessionReply {
    pub session_handle: u32,
    pub protocol_version: u16,
}

#[derive(Debug, Default)]
pub struct ListIdentityReply(pub Vec<IdentityObject>);
impl ListIdentityReply {
    #[inline(always)]
    pub fn into_inner(self) -> Vec<IdentityObject> {
        self.0
    }
}
