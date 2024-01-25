#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StreamTransactionsRequest {
    #[serde(
        rename = "at_ledger_state",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub at_ledger_state:
        Option<Option<Box<crate::models::LedgerStateSelector>>>,
    #[serde(
        rename = "from_ledger_state",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub from_ledger_state:
        Option<Option<Box<crate::models::LedgerStateSelector>>>,

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

    #[serde(rename = "kind_filter", skip_serializing_if = "Option::is_none")]
    pub kind_filter: Option<KindFilter>,
    #[serde(
        rename = "manifest_accounts_withdrawn_from_filter",
        skip_serializing_if = "Option::is_none"
    )]
    pub manifest_accounts_withdrawn_from_filter: Option<Vec<String>>,
    #[serde(
        rename = "manifest_accounts_deposited_into_filter",
        skip_serializing_if = "Option::is_none"
    )]
    pub manifest_accounts_deposited_into_filter: Option<Vec<String>>,
    #[serde(
        rename = "manifest_resources_filter",
        skip_serializing_if = "Option::is_none"
    )]
    pub manifest_resources_filter: Option<Vec<String>>,
    #[serde(
        rename = "affected_global_entities_filter",
        skip_serializing_if = "Option::is_none"
    )]
    pub affected_global_entities_filter: Option<Vec<String>>,
    #[serde(rename = "events_filter", skip_serializing_if = "Option::is_none")]
    pub events_filter:
        Option<Vec<crate::models::StreamTransactionsRequestEventFilterItem>>,

    #[serde(rename = "order", skip_serializing_if = "Option::is_none")]
    pub order: Option<Order>,
    #[serde(rename = "opt_ins", skip_serializing_if = "Option::is_none")]
    pub opt_ins: Option<Box<crate::models::TransactionDetailsOptIns>>,
}

impl Default for StreamTransactionsRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamTransactionsRequest {
    pub fn new() -> StreamTransactionsRequest {
        StreamTransactionsRequest {
            at_ledger_state: None,
            from_ledger_state: None,
            cursor: None,
            limit_per_page: None,
            kind_filter: None,
            manifest_accounts_withdrawn_from_filter: None,
            manifest_accounts_deposited_into_filter: None,
            manifest_resources_filter: None,
            affected_global_entities_filter: None,
            events_filter: None,
            order: None,
            opt_ins: None,
        }
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum KindFilter {
    #[serde(rename = "User")]
    User,
    #[serde(rename = "EpochChange")]
    EpochChange,
    #[serde(rename = "All")]
    All,
}

impl Default for KindFilter {
    fn default() -> KindFilter {
        Self::User
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Order {
    #[serde(rename = "Asc")]
    Asc,
    #[serde(rename = "Desc")]
    Desc,
}

impl Default for Order {
    fn default() -> Order {
        Self::Asc
    }
}
