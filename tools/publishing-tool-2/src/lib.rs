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

// TODO: Remove this crate.
//! This is a temporary tool that we will be using to deploy Ignition with
//! different user resources and it is a duplicate of the publishing-tool. The
//! main reason why it is a duplicate is because an approach that uses Rust
//! generics seems possible to achieve but difficult and the easiest way to do
//! this now would be to copy this crate.

pub mod configuration_selector;
pub mod database_overlay;
pub mod error;
pub mod macros;
pub mod network_connection_provider;
pub mod publishing;
pub mod utils;
