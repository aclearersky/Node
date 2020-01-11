// Copyright (c) 2017-2019, Substratum LLC (https://substratum.net) and/or its affiliates. All rights reserved.

use crate::sub_lib::wallet::Wallet;
use actix::Message;

#[derive(Clone, PartialEq, Debug)]
pub struct SetConsumingWalletMessage {
    pub wallet: Wallet,
}

impl Message for SetConsumingWalletMessage {
    type Result = ();
}
