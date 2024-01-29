#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EventsItem {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "emitter")]
    pub emitter: serde_json::Value,
    #[serde(rename = "data")]
    pub data: Box<serde_json::Value>,
}

impl EventsItem {
    pub fn new(
        name: String,
        emitter: serde_json::Value,
        data: serde_json::Value,
    ) -> EventsItem {
        EventsItem {
            name,
            emitter,
            data: Box::new(data),
        }
    }
}
