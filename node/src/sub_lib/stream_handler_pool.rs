// Copyright (c) 2017-2019, Substratum LLC (https://substratum.net) and/or its affiliates. All rights reserved.
use crate::sub_lib::dispatcher::Endpoint;
use crate::sub_lib::neighborhood::NodeQueryResponseMetadata;
use actix::Message;

#[derive(PartialEq, Debug, Clone)]
pub struct TransmitDataMsg {
    pub endpoint: Endpoint,
    pub last_data: bool,
    pub sequence_number: Option<u64>, // Some implies clear data; None implies clandestine.
    pub data: Vec<u8>,
}

impl Message for TransmitDataMsg {
    type Result = ();
}

#[derive(Clone)]
pub struct DispatcherNodeQueryResponse {
    pub result: Option<NodeQueryResponseMetadata>,
    pub context: TransmitDataMsg,
}

impl Message for DispatcherNodeQueryResponse {
    type Result = ();
}
