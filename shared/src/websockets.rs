use super::{ServerRequest, ServerResponse};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Message {
    Initialize,
    Reset,
    Message(WsBatchResponse),
    Connected,
}

pub trait ConnectedMessage
where
    Self: Sized,
{
    fn connected() -> Self;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WsRequest {
    pub id: i64,
    pub request: ServerRequest,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct WsBatchResponse {
    pub request_id: i64,
    pub results: Vec<ServerResponse>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WsResponse {
    pub request_id: i64,
    pub result: ServerResponse,
}
