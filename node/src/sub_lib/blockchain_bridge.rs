// Copyright (c) 2017-2019, Substratum LLC (https://substratum.net) and/or its affiliates. All rights reserved.

use crate::accountant::payable_dao::{PayableAccount, Payment};
use crate::blockchain::blockchain_bridge::RetrieveTransactions;
use crate::blockchain::blockchain_interface::BlockchainResult;
use crate::sub_lib::peer_actors::BindMessage;
use actix::Message;
use actix::Recipient;
use std::fmt;
use std::fmt::{Debug, Formatter};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct BlockchainBridgeConfig {
    pub blockchain_service_url: Option<String>,
    pub chain_id: u8,
    pub gas_price: Option<u64>,
}

#[derive(Clone)]
pub struct BlockchainBridgeSubs {
    pub bind: Recipient<BindMessage>,
    pub report_accounts_payable: Recipient<ReportAccountsPayable>,
    pub retrieve_transactions: Recipient<RetrieveTransactions>,
    pub set_consuming_db_password_sub: Recipient<SetDbPasswordMsg>,
    pub set_gas_price_sub: Recipient<SetGasPriceMsg>,
}

impl Debug for BlockchainBridgeSubs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "BlockchainBridgeSubs")
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ReportAccountsPayable {
    pub accounts: Vec<PayableAccount>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SetDbPasswordMsg {
    pub client_id: u64,
    pub password: String,
}

impl Message for SetDbPasswordMsg {
    type Result = ();
}

#[derive(Clone, PartialEq, Debug)]
pub struct SetGasPriceMsg {
    pub client_id: u64,
    pub gas_price: String,
}

impl Message for SetGasPriceMsg {
    type Result = ();
}

impl Message for ReportAccountsPayable {
    type Result = Result<Vec<BlockchainResult<Payment>>, String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::recorder::Recorder;
    use actix::Actor;

    #[test]
    fn blockchain_bridge_subs_debug() {
        let recorder = Recorder::new().start();

        let subject = BlockchainBridgeSubs {
            bind: recipient!(recorder, BindMessage),
            report_accounts_payable: recipient!(recorder, ReportAccountsPayable),
            retrieve_transactions: recipient!(recorder, RetrieveTransactions),
            set_consuming_db_password_sub: recipient!(recorder, SetDbPasswordMsg),
            set_gas_price_sub: recipient!(recorder, SetGasPriceMsg),
        };

        assert_eq!(format!("{:?}", subject), "BlockchainBridgeSubs");
    }
}
