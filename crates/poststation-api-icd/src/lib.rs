use chrono::{DateTime, Local};
use postcard_rpc::{
    endpoints, host_client::SchemaReport, standard_icd::WireError, topics, Key, TopicDirection,
};
use postcard_schema::Schema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type DeviceDatas = Vec<DeviceData>;
pub type OptSchemaReport = Option<SchemaReport>;
pub type OptVecLog = Option<Vec<Log>>;
pub type OptVecTopicMsg = Option<Vec<TopicMsg>>;

endpoints! {
    list = RACK_ENDPOINTS;
    | EndpointTy            | RequestTy     | ResponseTy        | Path                          |
    | ----------            | ---------     | ----------        | ----                          |
    | GetDevicesEndpoint    | ()            | DeviceDatas       | "rack/devices/get"            |
    | GetSchemasEndpoint    | u64           | OptSchemaReport   | "rack/devices/schemas/get"    |
    | GetLogsEndpoint       | LogRequest    | OptVecLog         | "rack/devices/logs/get"       |
    | GetTopicsEndpoint     | TopicRequest  | OptVecTopicMsg    | "rack/devices/topics/get"     |
    | ProxyEndpoint         | ProxyRequest  | ProxyResponse     | "rack/devices/proxy"          |
}

topics! {
    list = RACK_TOPICS_IN;
    direction = TopicDirection::ToServer;
    | TopicTy               | MessageTy         | Path                      |
    | -------               | ---------         | ----                      |
}

topics! {
    list = RACK_TOPICS_OUT;
    direction = TopicDirection::ToClient;
    | TopicTy               | MessageTy         | Path                      |
    | -------               | ---------         | ----                      |
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, Schema)]
pub struct DeviceData {
    pub serial: u64,
    pub name: String,
    pub is_connected: bool,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, Schema)]
pub struct LogRequest {
    pub serial: u64,
    pub count: u32,
}

// TODO: now that postcard-schema has a Schema impl for Uuid we might
// not actually need this anymore
#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, Schema)]
pub struct Uuidv7(pub [u8; 16]);

impl From<Uuid> for Uuidv7 {
    fn from(value: Uuid) -> Self {
        Self(value.into_bytes())
    }
}

impl From<Uuidv7> for Uuid {
    fn from(val: Uuidv7) -> Self {
        Uuid::from_bytes(val.0)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, Schema)]
pub struct Log {
    pub uuidv7: Uuidv7,
    pub msg: String,
}

impl Uuidv7 {
    pub fn id_to_time(&self) -> DateTime<Local> {
        let uuid = Uuid::from_bytes(self.0);
        let ts = uuid.get_timestamp().unwrap();
        let (a, b) = ts.to_unix();
        DateTime::from_timestamp(a as i64, b).unwrap().into()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, Schema)]
pub struct TopicRequest {
    pub serial: u64,
    pub path: String,
    pub key: Key,
    pub count: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, Schema)]
pub struct TopicMsg {
    pub uuidv7: Uuidv7,
    pub msg: Vec<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Schema)]
pub struct ProxyRequest {
    pub serial: u64,
    pub path: String,
    pub req_key: Key,
    pub resp_key: Key,
    pub seq_no: u32,
    pub req_body: Vec<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Schema)]
pub enum ProxyResponse {
    Ok {
        resp_key: Key,
        seq_no: u32,
        body: Vec<u8>,
    },
    WireErr {
        resp_key: Key,
        seq_no: u32,
        body: WireError,
    },
    OtherErr(String),
}