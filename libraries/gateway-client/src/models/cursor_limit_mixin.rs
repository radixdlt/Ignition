#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CursorLimitMixin {
    #[serde(
        rename = "cursor",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub cursor: Option<Option<String>>,

    #[serde(
        rename = "limit_per_page",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub limit_per_page: Option<Option<i32>>,
}

impl Default for CursorLimitMixin {
    fn default() -> Self {
        Self::new()
    }
}

impl CursorLimitMixin {
    pub fn new() -> CursorLimitMixin {
        CursorLimitMixin {
            cursor: None,
            limit_per_page: None,
        }
    }
}
