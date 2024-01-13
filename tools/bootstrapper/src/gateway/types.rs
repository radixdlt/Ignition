use radix_engine_interface::prelude::*;
use serde::*;
use serde_with::*;

pub(crate) trait Request {
    const ENDPOINT: &'static str;
    type Input: serde::Serialize;
    type Output: serde::de::DeserializeOwned;
}

pub(crate) mod status_gateway_status {
    use super::*;
    use common::*;

    pub const ENDPOINT: &str = "/status/gateway-status";

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Input;

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Output {
        pub ledger_state: LedgerState,
        pub release_info: ReleaseInfo,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ReleaseInfo {
        pub release_version: String,
        pub open_api_schema_version: String,
        pub image_tag: String,
    }

    pub struct Request;
    impl super::Request for Request {
        const ENDPOINT: &'static str = ENDPOINT;
        type Input = Input;
        type Output = Output;
    }
}

pub(crate) mod transaction_submit {
    use super::*;

    pub const ENDPOINT: &str = "/transaction/submit";

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Input {
        pub notarized_transaction_hex: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Output {
        pub duplicate: bool,
    }

    pub struct Request;
    impl super::Request for Request {
        const ENDPOINT: &'static str = ENDPOINT;
        type Input = Input;
        type Output = Output;
    }
}

pub(crate) mod transaction_committed_details {
    use super::*;

    pub const ENDPOINT: &str = "/transaction/committed-details";

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Input {
        pub intent_hash: String,
        pub opt_ins: OptIns,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct OptIns {
        pub raw_hex: bool,
        pub receipt_state_changes: bool,
        pub receipt_fee_summary: bool,
        pub receipt_fee_source: bool,
        pub receipt_fee_destination: bool,
        pub receipt_costing_parameters: bool,
        pub receipt_events: bool,
        pub affected_global_entities: bool,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Output {
        pub transaction: Transaction,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Transaction {
        pub transaction_status: TransactionStatus,
        pub receipt: Receipt,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub enum TransactionStatus {
        Unknown,
        CommittedSuccess,
        CommittedFailure,
        Pending,
        Rejected,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Receipt {
        pub state_updates: StateUpdates,
        pub events: Vec<Event>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Event {
        pub name: String,
        pub emitter: Emitter,
        pub data: serde_json::Value,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(tag = "type")]
    pub enum Emitter {
        Method {
            entity: Entity,
            object_module_id: String,
        },
        Function {
            blueprint_name: String,
            package_address: String,
        },
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct StateUpdates {
        pub new_global_entities: Vec<Entity>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Entity {
        pub is_global: bool,
        pub entity_type: String,
        pub entity_address: String,
    }

    pub struct Request;
    impl super::Request for Request {
        const ENDPOINT: &'static str = ENDPOINT;
        type Input = Input;
        type Output = Output;
    }
}

pub(crate) mod common {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct LedgerState {
        pub network: String,
        pub state_version: u64,
        pub proposer_round_timestamp: String,
        pub epoch: u64,
        pub round: u64,
    }
}
