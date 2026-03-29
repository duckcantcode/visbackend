use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Incoming {
    #[serde(rename = "type")]
    pub _type: String,

    // heartbeat

    // song change
    pub song_path: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct OutgoingSongInfo {
    pub fft: Vec<Vec<f32>>,
    pub period: f32,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Outgoing {
    #[serde(rename = "type")]
    pub _type: String,

    // heartbeat

    // song change
    pub song_info: Option<OutgoingSongInfo>,
}
