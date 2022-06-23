use crate::polkadot::RuntimeApi;
use codec::Encode;
use pallet_transaction_payment::FeeDetails;
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use subxt::rpc::ClientT;
use subxt::{rpc::rpc_params, sp_core::Bytes, BasicError, DefaultConfig, PolkadotExtrinsicParams};

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum NumberOrHex {
    Number(u64),
    Hex(U256),
}

impl NumberOrHex {
    pub fn as_u128(&self) -> u128 {
        match self {
            NumberOrHex::Number(number) => *number as u128,
            NumberOrHex::Hex(hex) => hex.as_u128(),
        }
    }
}

impl Default for NumberOrHex {
    fn default() -> Self {
        Self::Number(Default::default())
    }
}

pub async fn query_fee_details<Encodable: Encode>(
    api: &RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>,
    encoded_xt: &Encodable,
    at: Option<subxt::sp_core::H256>,
) -> Result<FeeDetails<NumberOrHex>, BasicError> {
    let encoded_xt: Bytes = encoded_xt.encode().into();

    let fee = api
        .client
        .rpc()
        .client
        .request("payment_queryFeeDetails", rpc_params![encoded_xt, at])
        .await?;

    Ok(fee)
}
