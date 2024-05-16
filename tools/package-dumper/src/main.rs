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

#![allow(clippy::arithmetic_side_effects)]

use clap::Parser;
use flate2::write::GzEncoder;
use flate2::Compression;
use radix_common::prelude::*;
use radix_engine_interface::prelude::*;
use radix_substate_store_interface::db_key_mapper::*;
use radix_substate_store_interface::interface::*;
use state_manager::store::traits::*;
use state_manager::store::*;
use std::io::Write;
use std::path::*;

type BranchStore =
    HashMap<DbNodeKey, HashMap<DbPartitionNum, HashMap<DbSortKey, Vec<u8>>>>;

const IGNORE_LIST: &[NodeId] = &[
    XRD.into_node_id(),
    SECP256K1_SIGNATURE_VIRTUAL_BADGE.into_node_id(),
    ED25519_SIGNATURE_VIRTUAL_BADGE.into_node_id(),
    PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.into_node_id(),
    GLOBAL_CALLER_VIRTUAL_BADGE.into_node_id(),
    SYSTEM_TRANSACTION_BADGE.into_node_id(),
    PACKAGE_OWNER_BADGE.into_node_id(),
    VALIDATOR_OWNER_BADGE.into_node_id(),
    ACCOUNT_OWNER_BADGE.into_node_id(),
    IDENTITY_OWNER_BADGE.into_node_id(),
    PACKAGE_PACKAGE.into_node_id(),
    RESOURCE_PACKAGE.into_node_id(),
    ACCOUNT_PACKAGE.into_node_id(),
    IDENTITY_PACKAGE.into_node_id(),
    CONSENSUS_MANAGER_PACKAGE.into_node_id(),
    ACCESS_CONTROLLER_PACKAGE.into_node_id(),
    POOL_PACKAGE.into_node_id(),
    TRANSACTION_PROCESSOR_PACKAGE.into_node_id(),
    METADATA_MODULE_PACKAGE.into_node_id(),
    ROYALTY_MODULE_PACKAGE.into_node_id(),
    ROLE_ASSIGNMENT_MODULE_PACKAGE.into_node_id(),
    TEST_UTILS_PACKAGE.into_node_id(),
    GENESIS_HELPER_PACKAGE.into_node_id(),
    FAUCET_PACKAGE.into_node_id(),
    TRANSACTION_TRACKER_PACKAGE.into_node_id(),
    CONSENSUS_MANAGER.into_node_id(),
    GENESIS_HELPER.into_node_id(),
    FAUCET.into_node_id(),
    TRANSACTION_TRACKER.into_node_id(),
];

/// Dumps the substates of a branch of the state tree and all of the nodes encountered..
#[derive(Parser, Debug)]
pub struct DumpStateBranch {
    /// The path to the state manager database.
    state_manager_database_dir: PathBuf,

    /// The Bech32m address of the packages to dump. Specifying multiple
    /// [`NodeId`]s means that the user wishes to get _multiple_ branches of the
    /// tree. This can help save space if the two branches have some common
    /// nodes.
    package_addresses: Vec<String>,

    /// The path to output the branches to. If not provided then the output path
    /// will be the concatenation of the root nodes separated by commas with
    /// `.bin` as the extension.
    #[arg(short, long)]
    out: Option<PathBuf>,

    /// Controls whether the output should be compressed or not. If this flag
    /// is added, then the output will be gz compressed.
    #[arg(short, long, default_value_t = false)]
    compress: bool,

    /// Controls whether this command should be verbose or not. If it is run in
    /// verbose mode, then the tree it printed to stdout.
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// The id of the network that the database belongs to. This is used for the
    /// encoding of the encountered addresses. Defaults to 0x01 (mainnet) if not
    /// provided.
    #[arg(short, long, default_value_t = 1)]
    network_id: u8,
}

fn main() -> Result<(), Error> {
    let DumpStateBranch {
        state_manager_database_dir,
        package_addresses,
        out,
        compress,
        verbose,
        network_id,
    } = DumpStateBranch::parse();

    // Prepare the network dependent objects.
    let network_definition = network_definition_from_network_id(network_id);
    let db = ActualStateManagerDatabase::new(
        state_manager_database_dir.clone(),
        DatabaseConfig::default(),
        &network_definition,
    )
    .map_err(|_| Error::FailedToLoadDatabase)?;
    let decoder = AddressBech32Decoder::new(&network_definition);
    let encoder = AddressBech32Encoder::new(&network_definition);

    // Stores the branches and the nodes requested in order.
    let mut branches = (Vec::<NodeId>::new(), BranchStore::new());

    // Construct the list of nodes to ignore. These are nodes who are published
    // or created as part of genesis and thus do not need to be included in the
    // state branch.
    let ignore_list = IGNORE_LIST
        .iter()
        .map(SpreadPrefixKeyMapper::to_db_node_key)
        .collect();

    for node_id in package_addresses.iter() {
        // Decode the node-id string and then add it to the list of nodes.
        let package_address =
            PackageAddress::try_from_bech32(&decoder, node_id)
                .ok_or(Error::InvalidAddress)?;
        branches.0.push(package_address.into_node_id());

        // Traverse the state tree and add state to the branch store.
        traverse_tree(
            &SpreadPrefixKeyMapper::to_db_node_key(
                &package_address.into_node_id(),
            ),
            &db,
            &mut branches.1,
            &ignore_list,
            &encoder,
            0,
            verbose,
        )?;
    }

    let database_updates = DatabaseUpdates {
        node_updates: branches
            .1
            .into_iter()
            .map(|(db_node_key, partition_num_to_updates_mapping)| {
                (
                    db_node_key,
                    NodeDatabaseUpdates {
                        partition_updates: partition_num_to_updates_mapping
                            .into_iter()
                            .map(|(partition_num, substates)| {
                                (
                                    partition_num,
                                    PartitionDatabaseUpdates::Delta {
                                        substate_updates: substates
                                            .into_iter()
                                            .map(|(db_sort_key, value)| {
                                                (
                                                    db_sort_key,
                                                    DatabaseUpdate::Set(value),
                                                )
                                            })
                                            .collect(),
                                    },
                                )
                            })
                            .collect(),
                    },
                )
            })
            .collect(),
    };

    let output_path = out
        .clone()
        .unwrap_or(format!("{}.bin", package_addresses.join(",")).into());

    let encoded = scrypto_encode(&database_updates).expect("Can't fail!");
    let output = if compress {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&encoded).map_err(Error::IOError)?;
        encoder.finish().map_err(Error::IOError)?
    } else {
        encoded
    };
    std::fs::write(output_path, output).map_err(Error::IOError)?;

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    FailedToLoadDatabase,
    InvalidAddress,
    IOError(std::io::Error),
}

pub fn network_definition_from_network_id(network_id: u8) -> NetworkDefinition {
    match network_id {
        // Public facing networks
        0x01 => NetworkDefinition::mainnet(),
        0x02 => NetworkDefinition {
            id: network_id,
            logical_name: "stokenet".to_owned(),
            hrp_suffix: "tdx_2_".to_owned(),
        },

        // Babylon Temporary Testnets
        0x0A => NetworkDefinition::adapanet(),
        0x0B => NetworkDefinition::nebunet(),
        0x0C => NetworkDefinition {
            id: network_id,
            logical_name: "kisharnet".to_owned(),
            hrp_suffix: "tdx_c_".to_owned(),
        },
        0x0D => NetworkDefinition {
            id: network_id,
            logical_name: "ansharnet".to_owned(),
            hrp_suffix: "tdx_d_".to_owned(),
        },

        // RDX Works Development
        0x20 => NetworkDefinition {
            id: 0x20,
            logical_name: "gilganet".to_owned(),
            hrp_suffix: "tdx_20_".to_owned(),
        },
        0x21 => NetworkDefinition {
            id: 0x21,
            logical_name: "enkinet".to_owned(),
            hrp_suffix: "tdx_21_".to_owned(),
        },
        0x22 => NetworkDefinition {
            id: 0x22,
            logical_name: "hammunet".to_owned(),
            hrp_suffix: "tdx_22_".to_owned(),
        },
        0x23 => NetworkDefinition {
            id: 0x23,
            logical_name: "nergalnet".to_owned(),
            hrp_suffix: "tdx_23_".to_owned(),
        },
        0x24 => NetworkDefinition {
            id: 0x24,
            logical_name: "mardunet".to_owned(),
            hrp_suffix: "tdx_24_".to_owned(),
        },
        0x25 => NetworkDefinition {
            id: 0x25,
            logical_name: "dumunet".to_owned(),
            hrp_suffix: "tdx_25_".to_owned(),
        },

        // Ephemeral Networks
        0xF0 => NetworkDefinition {
            id: 240,
            logical_name: "localnet".to_owned(),
            hrp_suffix: "loc".to_owned(),
        },
        0xF1 => NetworkDefinition {
            id: 241,
            logical_name: "inttestnet".to_owned(),
            hrp_suffix: "test".to_owned(),
        },
        0xF2 => NetworkDefinition::simulator(),

        // Unnamed
        network_id => NetworkDefinition {
            id: network_id,
            logical_name: "unnamed".to_owned(),
            hrp_suffix: format!("tdx_{:x}_", network_id),
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn traverse_tree<S>(
    node_id: &DbNodeKey,
    db: &S,
    store: &mut BranchStore,
    ignore_list: &HashSet<DbNodeKey>,
    /* Machinery for printing, not required for traversal */
    encoder: &AddressBech32Encoder,
    depth: usize,
    verbose: bool,
) -> Result<(), Error>
where
    S: SubstateDatabase + ListableSubstateDatabase,
{
    // If we're operating in verbose mode, then print out the node id prefixed with as many
    // spaces as the current depth (times 2 just to make it more readable).
    if verbose {
        println!(
            "{}{}",
            " ".repeat(depth * 2),
            encoder.encode(&db_node_key_to_node_id(node_id).0).unwrap()
        );
    }

    // Iterate over all of the partition for the given node and add their substates to the
    // branches store.
    (u8::MIN..=u8::MAX)
        .map(|partition_number| DbPartitionKey {
            node_key: node_id.clone(),
            partition_num: partition_number,
        })
        .for_each(|partition_key| {
            db.list_entries(&partition_key)
                .for_each(|(sort_key, value)| {
                    // Insert into the branches store.
                    store
                        .entry(partition_key.node_key.clone())
                        .or_default()
                        .entry(partition_key.partition_num)
                        .or_default()
                        .insert(sort_key, value);
                })
        });

    // Find all of the nodes referenced in the substates of this node - this is
    // so that we continue traversing this tree when we discover another node in
    // the tree that we have not yet looked at.
    let nodes_to_traverse = store
        .get(node_id)
        .expect("How come we have a node that has no partitions or substates?")
        .values()
        .flat_map(|item| item.values())
        .flat_map(|substate_value| {
            let indexed_value = IndexedScryptoValue::from_slice(substate_value)
                .expect("Substate store must contain valid substates");

            let mut node_ids = indexed_value.owned_nodes().clone();
            node_ids.extend(indexed_value.references());
            node_ids
        })
        .filter_map(|node_id| {
            let node_id = SpreadPrefixKeyMapper::to_db_node_key(&node_id);
            if store.contains_key(&node_id) || ignore_list.contains(&node_id) {
                None
            } else {
                Some(node_id)
            }
        })
        .collect::<HashSet<_>>();
    for db_node_key in nodes_to_traverse {
        traverse_tree(
            &db_node_key,
            db,
            store,
            ignore_list,
            encoder,
            depth + 1,
            verbose,
        )?;
    }

    Ok(())
}

fn db_node_key_to_node_id(node_key: &DbNodeKey) -> NodeId {
    // Sadly, the `SpreadPrefixKeyMapper` does not expose a way for us to just transform a
    // NodeKey into a NodeId. So, we must create a partition key first and then perform the
    // transformation.
    SpreadPrefixKeyMapper::from_db_partition_key(&DbPartitionKey {
        node_key: node_key.clone(),
        partition_num: 0,
    })
    .0
}
