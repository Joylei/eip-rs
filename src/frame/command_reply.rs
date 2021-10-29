use crate::objects::{identity::IdentityObject, service::ListServiceItem};

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

#[derive(Debug, Default)]
pub struct ListServicesReply(pub Vec<ListServiceItem>);
impl ListServicesReply {
    #[inline(always)]
    pub fn into_inner(self) -> Vec<ListServiceItem> {
        self.0
    }
}

pub struct SendRRDataReply;

pub struct SendUnitDataReply;
