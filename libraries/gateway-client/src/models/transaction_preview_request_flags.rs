#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionPreviewRequestFlags {
    #[serde(rename = "use_free_credit")]
    pub use_free_credit: bool,
    #[serde(rename = "assume_all_signature_proofs")]
    pub assume_all_signature_proofs: bool,
    #[serde(rename = "skip_epoch_check")]
    pub skip_epoch_check: bool,
}

impl TransactionPreviewRequestFlags {
    pub fn new(
        use_free_credit: bool,
        assume_all_signature_proofs: bool,
        skip_epoch_check: bool,
    ) -> TransactionPreviewRequestFlags {
        TransactionPreviewRequestFlags {
            use_free_credit,
            assume_all_signature_proofs,
            skip_epoch_check,
        }
    }
}
