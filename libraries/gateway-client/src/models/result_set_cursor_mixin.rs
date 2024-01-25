#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResultSetCursorMixin {
    #[serde(
        rename = "total_count",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub total_count: Option<Option<i64>>,

    #[serde(
        rename = "next_cursor",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub next_cursor: Option<Option<String>>,
}

impl Default for ResultSetCursorMixin {
    fn default() -> Self {
        Self::new()
    }
}

impl ResultSetCursorMixin {
    pub fn new() -> ResultSetCursorMixin {
        ResultSetCursorMixin {
            total_count: None,
            next_cursor: None,
        }
    }
}
