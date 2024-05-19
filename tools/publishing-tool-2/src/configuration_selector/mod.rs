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

mod mainnet_production;

use crate::publishing::*;
use clap::*;
use radix_common::prelude::*;
use radix_transactions::prelude::*;

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum ConfigurationSelector {
    MainnetProduction,
}

impl ConfigurationSelector {
    pub fn configuration(
        self,
        notary_private_key: &PrivateKey,
    ) -> PublishingConfiguration {
        match self {
            Self::MainnetProduction => {
                mainnet_production::mainnet_production(notary_private_key)
            }
        }
    }

    pub fn gateway_base_url(self) -> String {
        match self {
            Self::MainnetProduction => {
                "https://mainnet.radixdlt.com".to_owned()
            }
        }
    }

    pub fn network_definition(self) -> NetworkDefinition {
        match self {
            Self::MainnetProduction => NetworkDefinition::mainnet(),
        }
    }
}
