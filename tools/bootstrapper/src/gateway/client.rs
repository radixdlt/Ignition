use radix_engine_interface::prelude::*;
use transaction::prelude::*;

use reqwest::blocking::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GatewayClient {
    base_url: String,
}

impl GatewayClient {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            base_url: url.into(),
        }
    }

    pub fn stokenet() -> Self {
        Self::new(crate::constants::STOKENET_GATEWAY_URL)
    }

    pub fn get_current_epoch(&self) -> reqwest::Result<Epoch> {
        self.make_request::<super::types::status_gateway_status::Request>(
            &super::types::status_gateway_status::Input,
        )
        .map(|response| Epoch::of(response.ledger_state.epoch))
    }

    pub fn submit_transaction(
        &self,
        transaction: &NotarizedTransactionV1,
    ) -> reqwest::Result<bool> {
        let bytes = transaction.to_payload_bytes().expect("Can't happen!");
        self.make_request::<super::types::transaction_submit::Request>(
            &super::types::transaction_submit::Input {
                notarized_transaction_hex: hex::encode(bytes),
            },
        )
        .map(|response| response.duplicate)
    }

    pub fn transaction_committed_details(
        &self,
        bech32m_intent_hash: String,
    ) -> reqwest::Result<super::types::transaction_committed_details::Output>
    {
        self.make_request::<super::types::transaction_committed_details::Request>(
            &super::types::transaction_committed_details::Input {
                intent_hash: bech32m_intent_hash,
                opt_ins: super::types::transaction_committed_details::OptIns {
                    raw_hex: true,
                    receipt_state_changes: true,
                    receipt_fee_summary: true,
                    receipt_fee_source: true,
                    receipt_fee_destination: true,
                    receipt_costing_parameters: true,
                    receipt_events: true,
                    affected_global_entities: true,
                }
            },
        )
    }

    fn make_request<R: super::types::Request>(
        &self,
        input: &R::Input,
    ) -> Result<R::Output, reqwest::Error> {
        Client::new()
            .post(format!("{}/{}", self.base_url, R::ENDPOINT))
            .json(input)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36 Edg/118.0.2088.46")
            .send()
            .and_then(|response| response.json())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_get_current_epoch() {
        // Arrange
        let client = GatewayClient::stokenet();

        // Act
        let epoch = dbg!(client.get_current_epoch());

        // Assert
        assert!(epoch.is_ok())
    }
}
