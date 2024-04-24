// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use radix_engine::transaction::TransactionReceiptV1;
use transaction::prelude::*;

/// A standardized interface for objects that provide connection to the network
/// regardless of how these objects are implemented and how they provide such
/// connection. One implementation could choose to provide network connection
/// through the core-api, another might do it over the gateway-api, and another
/// might talk directly to a node. The implementation details are abstracted
/// away in the interface. The interface has a number of getter functions and
/// functions for executing transactions.
pub trait NetworkConnectionProvider {
    type Error: Debug;

    fn execute_transaction(
        &mut self,
        notarized_transaction: &NotarizedTransactionV1,
    ) -> Result<ExecutionReceipt, Self::Error>;

    fn preview_transaction(
        &mut self,
        preview_intent: PreviewIntentV1,
    ) -> Result<TransactionReceiptV1, Self::Error>;

    fn get_current_epoch(&mut self) -> Result<Epoch, Self::Error>;

    fn get_network_definition(
        &mut self,
    ) -> Result<NetworkDefinition, Self::Error>;

    fn read_component_state<V: ScryptoDecode>(
        &mut self,
        component_address: ComponentAddress,
    ) -> Result<V, Self::Error>;
}

/// A simplified transaction receipt containing the key pieces of information
/// that must be included in an execution receipt. This is limited by the data
/// that the node can give us.
#[derive(Clone, Debug, PartialEq, Eq, ScryptoSbor)]
pub enum ExecutionReceipt {
    CommitSuccess(ExecutionReceiptSuccessContents),
    CommitFailure { reason: String },
    Rejection { reason: String },
    Abort { reason: String },
}

#[derive(Clone, Default, Debug, PartialEq, Eq, ScryptoSbor)]
pub struct NewEntities {
    pub new_component_addresses: IndexSet<ComponentAddress>,
    pub new_resource_addresses: IndexSet<ResourceAddress>,
    pub new_package_addresses: IndexSet<PackageAddress>,
}

#[derive(Clone, Debug, PartialEq, Eq, ScryptoSbor)]
pub struct ExecutionReceiptSuccessContents {
    pub new_entities: NewEntities,
}
