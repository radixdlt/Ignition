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

#![allow(unused_macros)]

#[macro_export]
macro_rules! package_address {
    ($address: expr) => {
        ::radix_engine_interface::prelude::PackageAddress::try_from(
            $crate::decode_to_node_id!($address),
        )
        .unwrap()
    };
}

#[macro_export]
macro_rules! component_address {
    ($address: expr) => {
        ::radix_engine_interface::prelude::ComponentAddress::try_from(
            $crate::decode_to_node_id!($address),
        )
        .unwrap()
    };
}

#[macro_export]
macro_rules! resource_address {
    ($address: expr) => {
        ::radix_engine_interface::prelude::ResourceAddress::try_from(
            $crate::decode_to_node_id!($address),
        )
        .unwrap()
    };
}

#[macro_export]
macro_rules! internal_address {
    ($address: expr) => {
        ::radix_engine_interface::prelude::InternalAddress::try_from(
            $crate::decode_to_node_id!($address),
        )
        .unwrap()
    };
}

#[macro_export]
macro_rules! global_address {
    ($address: expr) => {
        ::radix_engine_interface::prelude::GlobalAddress::try_from(
            $crate::decode_to_node_id!($address),
        )
        .unwrap()
    };
}

#[macro_export]
macro_rules! decode_to_node_id {
    ($address: expr) => {
        ::radix_common::prelude::AddressBech32Decoder::validate_and_decode_ignore_hrp(
            $address,
        )
        .ok()
        .and_then(|(_, _, value)| value.try_into().map(NodeId).ok())
        .unwrap()
    };
}
