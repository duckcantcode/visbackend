use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Incoming {
    #[serde(rename = "type")]
    pub _type: String,

    // heartbeat

    // song change
    pub song_path: Option<String>,
}
