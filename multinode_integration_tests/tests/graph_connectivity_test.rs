// Copyright (c) 2017-2019, Substratum LLC (https://substratum.net) and/or its affiliates. All rights reserved.

use multinode_integration_tests_lib::substratum_node::SubstratumNode;
use multinode_integration_tests_lib::substratum_node_cluster::SubstratumNodeCluster;
use multinode_integration_tests_lib::substratum_real_node::{
    NodeStartupConfigBuilder, SubstratumRealNode,
};
use node_lib::neighborhood::gossip::{Gossip, GossipNodeRecord};
use node_lib::neighborhood::node_record::{NodeRecordInner, NodeSignatures};
use node_lib::sub_lib::cryptde::{CryptData, PublicKey};
use node_lib::sub_lib::neighborhood::DEFAULT_RATE_PACK;
use node_lib::sub_lib::wallet::Wallet;
use node_lib::test_utils::test_utils::{find_free_port, vec_to_set};
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

#[test]
fn graph_connects_but_does_not_over_connect() {
    let neighborhood_size = 5;
    let mut cluster = SubstratumNodeCluster::start().unwrap();

    let bootstrap_node = cluster.start_real_node(NodeStartupConfigBuilder::bootstrap().build());
    let real_nodes = (0..neighborhood_size)
        .map(|_| {
            cluster.start_real_node(
                NodeStartupConfigBuilder::standard()
                    .neighbor(bootstrap_node.node_reference())
                    .build(),
            )
        })
        .collect::<Vec<SubstratumRealNode>>();
    let mock_node = cluster.start_mock_node(vec![find_free_port()]);
    let dont_count_these = vec![bootstrap_node.public_key(), mock_node.public_key()];
    let start_node = real_nodes.first().unwrap();
    // Wait for Gossip to abate
    thread::sleep(Duration::from_millis(2000));

    // Start the bootstrap process; follow passes until Introductions arrive
    mock_node.send_debut(start_node);
    let mut retries_left = neighborhood_size;
    let mut introductions_opt: Option<Gossip> = None;
    while retries_left > 0 {
        let (intros, _) = mock_node
            .wait_for_gossip(Duration::from_millis(1000))
            .unwrap();
        if intros.node_records.len() > 1 {
            introductions_opt = Some(intros);
            break;
        }
        let pass_target = real_nodes
            .iter()
            .find(|n| n.public_key() == intros.node_records[0].public_key())
            .unwrap();
        mock_node.send_debut(pass_target);
        retries_left -= 1;
    }
    let introductions = introductions_opt.unwrap();

    // Compose and send a standard Gossip message that will stimulate a general Gossip broadcast
    let another_gnr = introductions
        .node_records
        .iter()
        .find(|gnr| gnr.public_key() != mock_node.public_key())
        .unwrap();
    let mock_gnr = GossipNodeRecord {
        inner: NodeRecordInner {
            public_key: mock_node.public_key(),
            node_addr_opt: Some(mock_node.node_addr()),
            earning_wallet: Wallet::new("0000"),
            consuming_wallet: None,
            rate_pack: DEFAULT_RATE_PACK.clone(),
            is_bootstrap_node: false,
            neighbors: vec_to_set(vec![start_node.public_key()]),
            version: 100, // to make the sample Node update its database and send out standard Gossip
        },
        signatures: NodeSignatures {
            complete: CryptData::new(b""),
            obscured: CryptData::new(b""),
        },
    };
    let standard_gossip = Gossip {
        node_records: vec![mock_gnr.clone(), another_gnr.clone()],
    };
    let socket_addrs: Vec<SocketAddr> = start_node.node_addr().into();
    mock_node
        .transmit_gossip(
            mock_node.port_list()[0],
            standard_gossip,
            &start_node.public_key(),
            socket_addrs[0],
        )
        .unwrap();

    // Snag the broadcast and assert on it: everything that isn't test harness or bootstrap Node
    // should have degree at least 2 and no more than 5.
    let (current_state, _) = mock_node
        .wait_for_gossip(Duration::from_millis(1000))
        .unwrap();
    let dot_graph =
        current_state.to_dot_graph(&another_gnr.to_node_record(), &mock_gnr.to_node_record());
    // True number of Nodes in source database should be neighborhood_size + 2,
    // but gossip target (mock_node) will not be included in Gossip so should be neighborhood size + 1 (bootstrap).
    assert_eq!(
        neighborhood_size + 1,
        current_state.node_records.len(),
        "Current-state Gossip should have {} GossipNodeRecords, but has {}: {}",
        neighborhood_size + 1,
        current_state.node_records.len(),
        dot_graph
    );
    let key_degrees = current_state
        .node_records
        .iter()
        .filter(|gnr| !dont_count_these.contains(&gnr.public_key()))
        .map(|gnr| {
            (
                gnr.public_key(),
                degree(&current_state, &gnr.public_key(), &dont_count_these),
            )
        })
        .filter(|pair| (pair.1 < 2 || pair.1 > 5))
        .collect::<Vec<(PublicKey, usize)>>();
    assert!(
        key_degrees.is_empty(),
        "These Nodes had the wrong number of neighbors: {:?}\n{}",
        key_degrees,
        dot_graph
    );
}

fn degree(gossip: &Gossip, key: &PublicKey, dont_count_these: &Vec<PublicKey>) -> usize {
    record_of(gossip, key)
        .unwrap()
        .inner
        .neighbors
        .iter()
        .filter(|k| !dont_count_these.contains(*k))
        .count()
}

fn record_of<'a>(gossip: &'a Gossip, key: &PublicKey) -> Option<&'a GossipNodeRecord> {
    gossip.node_records.iter().find(|n| &n.public_key() == key)
}
