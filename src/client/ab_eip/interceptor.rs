use super::HasMore;
use rseip_cip::{
    codec::decode::message_reply::decode_service_and_status, error::cip_error_status, MessageReply,
    MessageReplyInterface, Status,
};
use rseip_core::codec::{Decode, Decoder};

#[derive(Debug)]
pub(crate) struct HasMoreInterceptor<T>(pub MessageReply<T>);

impl<T> MessageReplyInterface for HasMoreInterceptor<T> {
    type Value = T;

    fn reply_service(&self) -> u8 {
        self.0.reply_service
    }

    fn status(&self) -> &Status {
        &self.0.status
    }

    fn value(&self) -> &Self::Value {
        &self.0.data
    }

    fn into_value(self) -> Self::Value {
        self.0.data
    }
}

impl<'de, T> Decode<'de> for HasMoreInterceptor<T>
where
    T: Decode<'de>,
{
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let (reply_service, status) = decode_service_and_status(&mut decoder)?;
        if status.is_err() && !status.has_more() {
            return Err(cip_error_status(status));
        }
        let data = decoder.decode_any()?;
        Ok(Self(MessageReply::new(reply_service, status, data)))
    }
}
