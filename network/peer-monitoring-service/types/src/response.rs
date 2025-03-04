// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use aptos_config::{config::PeerRole, network_id::PeerNetworkId};
use aptos_types::{network_address::NetworkAddress, PeerId};
use cfg_block::cfg_block;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Duration};
use thiserror::Error;

/// A peer monitoring service response
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum PeerMonitoringServiceResponse {
    LatencyPing(LatencyPingResponse), // A simple message to respond to latency checks (i.e., pings)
    NetworkInformation(NetworkInformationResponse), // Holds the response for network information
    NodeInformation(NodeInformationResponse), // Holds the response for node information
    ServerProtocolVersion(ServerProtocolVersionResponse), // Returns the current server protocol version

    #[cfg(feature = "network-perf-test")] // Disabled by default
    PerformanceMonitoring(PerformanceMonitoringResponse), // A response for performance monitoring requests
}

impl PeerMonitoringServiceResponse {
    /// Returns a summary label for the response
    pub fn get_label(&self) -> &'static str {
        match self {
            Self::LatencyPing(_) => "latency_ping",
            Self::NetworkInformation(_) => "network_information",
            Self::NodeInformation(_) => "node_information",
            Self::ServerProtocolVersion(_) => "server_protocol_version",

            #[cfg(feature = "network-perf-test")] // Disabled by default
            Self::PerformanceMonitoring(_) => "performance_monitoring_response",
        }
    }

    /// Returns the number of bytes in the serialized response
    pub fn get_num_bytes(&self) -> Result<u64, UnexpectedResponseError> {
        let serialized_bytes = bcs::to_bytes(&self).map_err(|error| {
            UnexpectedResponseError(format!(
                "Failed to serialize response: {}. Error: {:?}",
                self.get_label(),
                error
            ))
        })?;
        Ok(serialized_bytes.len() as u64)
    }
}

/// A response for the latency ping request
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LatencyPingResponse {
    pub ping_counter: u64, // A monotonically increasing counter to verify latency ping responses
}

/// A response for the network information request
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NetworkInformationResponse {
    pub connected_peers: BTreeMap<PeerNetworkId, ConnectionMetadata>, // Connected peers
    pub distance_from_validators: u64, // The distance of the peer from the validator set
}

/// Simple connection metadata associated with each peer
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ConnectionMetadata {
    pub network_address: NetworkAddress,
    pub peer_id: PeerId,
    pub peer_role: PeerRole,
}

impl ConnectionMetadata {
    pub fn new(network_address: NetworkAddress, peer_id: PeerId, peer_role: PeerRole) -> Self {
        Self {
            network_address,
            peer_id,
            peer_role,
        }
    }
}

/// A response for the server protocol version request
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ServerProtocolVersionResponse {
    pub version: u64, // The version of the peer monitoring service run by the server
}

/// A response for the node information request
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NodeInformationResponse {
    pub build_information: BTreeMap<String, String>, // The build information of the node
    pub highest_synced_epoch: u64,                   // The highest synced epoch of the node
    pub highest_synced_version: u64,                 // The highest synced version of the node
    pub ledger_timestamp_usecs: u64, // The latest timestamp of the blockchain (in microseconds)
    pub lowest_available_version: u64, // The lowest stored version of the node (in storage)
    pub uptime: Duration,            // The amount of time the peer has been running
}

#[derive(Clone, Debug, Error)]
#[error("Unexpected response variant: {0}")]
pub struct UnexpectedResponseError(pub String);

impl TryFrom<PeerMonitoringServiceResponse> for LatencyPingResponse {
    type Error = UnexpectedResponseError;

    fn try_from(response: PeerMonitoringServiceResponse) -> crate::Result<Self, Self::Error> {
        match response {
            PeerMonitoringServiceResponse::LatencyPing(inner) => Ok(inner),
            _ => Err(UnexpectedResponseError(format!(
                "expected latency_ping_response, found {}",
                response.get_label()
            ))),
        }
    }
}

impl TryFrom<PeerMonitoringServiceResponse> for NetworkInformationResponse {
    type Error = UnexpectedResponseError;

    fn try_from(response: PeerMonitoringServiceResponse) -> crate::Result<Self, Self::Error> {
        match response {
            PeerMonitoringServiceResponse::NetworkInformation(inner) => Ok(inner),
            _ => Err(UnexpectedResponseError(format!(
                "expected network_information_response, found {}",
                response.get_label()
            ))),
        }
    }
}

impl TryFrom<PeerMonitoringServiceResponse> for NodeInformationResponse {
    type Error = UnexpectedResponseError;

    fn try_from(response: PeerMonitoringServiceResponse) -> crate::Result<Self, Self::Error> {
        match response {
            PeerMonitoringServiceResponse::NodeInformation(inner) => Ok(inner),
            _ => Err(UnexpectedResponseError(format!(
                "expected node_information_response, found {}",
                response.get_label()
            ))),
        }
    }
}

impl TryFrom<PeerMonitoringServiceResponse> for ServerProtocolVersionResponse {
    type Error = UnexpectedResponseError;

    fn try_from(response: PeerMonitoringServiceResponse) -> crate::Result<Self, Self::Error> {
        match response {
            PeerMonitoringServiceResponse::ServerProtocolVersion(inner) => Ok(inner),
            _ => Err(UnexpectedResponseError(format!(
                "expected server_protocol_version_response, found {}",
                response.get_label()
            ))),
        }
    }
}

cfg_block! {
    #[cfg(feature = "network-perf-test")] { // Disabled by default
        /// A response for performance monitoring requests
        #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
        pub struct PerformanceMonitoringResponse {
            pub response_counter: u64, // A monotonically increasing counter to verify responses
        }

        impl TryFrom<PeerMonitoringServiceResponse> for PerformanceMonitoringResponse {
            type Error = UnexpectedResponseError;

            fn try_from(response: PeerMonitoringServiceResponse) -> crate::Result<Self, Self::Error> {
                match response {
                    PeerMonitoringServiceResponse::PerformanceMonitoring(inner) => Ok(inner),
                    _ => Err(UnexpectedResponseError(format!(
                        "expected performance_monitoring_response, found {}",
                        response.get_label()
                    ))),
                }
            }
        }
    }
}
