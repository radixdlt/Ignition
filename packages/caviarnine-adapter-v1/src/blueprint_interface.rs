use scrypto::prelude::*;
use scrypto_interface::*;

define_interface! {
    QuantaSwap as CaviarNinePool impl [
        ScryptoStub,
        ScryptoTestStub,
        #[cfg(feature = "manifest-builder-stubs")]
        ManifestBuilderStub
    ] {
        fn new(
            owner_rule: AccessRule,
            user_rule: AccessRule,
            token_x_address: ResourceAddress,
            token_y_address: ResourceAddress,
            bin_span: u32,
            #[manifest_type = "Option<ManifestAddressReservation>"]
            reservation: Option<GlobalAddressReservation>,
        ) -> Self;
        fn get_fee_controller_address(&self) -> ComponentAddress;
        fn get_fee_vaults_address(&self) -> ComponentAddress;
        fn get_token_x_address(&self) -> ResourceAddress;
        fn get_token_y_address(&self) -> ResourceAddress;
        fn get_liquidity_receipt_address(&self) -> ResourceAddress;
        fn get_bin_span(&self) -> u32;
        fn get_amount_x(&self) -> Decimal;
        fn get_amount_y(&self) -> Decimal;
        fn get_active_tick(&self) -> Option<u32>;
        fn get_price(&self) -> Option<Decimal>;
        fn get_active_bin_price_range(&self) -> Option<(Decimal, Decimal)>;
        fn get_active_amounts(&self) -> Option<(Decimal, Decimal)>;
        fn get_bins_above(
            &self,
            start_tick: Option<u32>,
            stop_tick: Option<u32>,
            number: Option<u32>,
        ) -> Vec<(u32, Decimal)>;
        fn get_bins_below(
            &self,
            start_tick: Option<u32>,
            stop_tick: Option<u32>,
            number: Option<u32>,
        ) -> Vec<(u32, Decimal)>;
        fn get_liquidity_claims(
            &self,
            liquidity_receipt_id: NonFungibleLocalId,
        ) -> IndexMap<u32, Decimal>;
        fn get_redemption_value(&self, liquidity_receipt_id: NonFungibleLocalId) -> (Decimal, Decimal);
        fn get_redemption_bin_values(
            &self,
            liquidity_receipt_id: NonFungibleLocalId,
        ) -> Vec<(u32, Decimal, Decimal)>;
        fn mint_liquidity_receipt(&mut self) -> Bucket;
        fn burn_liquidity_receipt(
            &mut self,
            #[manifest_type = "ManifestBucket"]
            liquidity_receipt: Bucket
        );
        fn add_liquidity_to_receipt(
            &mut self,
            #[manifest_type = "ManifestBucket"]
            liquidity_receipt: Bucket,
            #[manifest_type = "ManifestBucket"]
            tokens_x: Bucket,
            #[manifest_type = "ManifestBucket"]
            tokens_y: Bucket,
            positions: Vec<(u32, Decimal, Decimal)>,
        ) -> (Bucket, Bucket, Bucket);
        fn add_liquidity(
            &mut self,
            #[manifest_type = "ManifestBucket"]
            tokens_x: Bucket,
            #[manifest_type = "ManifestBucket"]
            tokens_y: Bucket,
            positions: Vec<(u32, Decimal, Decimal)>,
        ) -> (Bucket, Bucket, Bucket);
        fn remove_specific_liquidity(
            &mut self,
            #[manifest_type = "ManifestBucket"]
            liquidity_receipt: Bucket,
            claims: Vec<(u32, Decimal)>,
        ) -> (Bucket, Bucket, Bucket);
        fn remove_liquidity(
            &mut self,
            #[manifest_type = "ManifestBucket"]
            liquidity_receipt: Bucket
        ) -> (Bucket, Bucket);
        fn swap(
            &mut self,
            #[manifest_type = "ManifestBucket"]
            tokens: Bucket
        ) -> (Bucket, Bucket);
    }
}
