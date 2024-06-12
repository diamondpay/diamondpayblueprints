use scrypto::prelude::*;

// DAPP_ACCOUNT needs to be updated before deploy
pub const INFO_URL: &str = "https://github.com/diamondpay";
pub const ICON_URL: &str = "https://avatars.githubusercontent.com/u/162780104";

#[derive(NonFungibleData, ScryptoSbor)]
pub struct BadgeData {
    pub contract_address: ComponentAddress,
    pub contract_handle: String,
    pub app_handle: String,
    pub member_handle: String,
}

#[derive(ScryptoSbor, Clone)]
pub enum TxType {
    Create,
    Deposit,
    Invite,
    Remove,
    Leave,
    Update,
    Join,
    Reward,
    Withdraw,
    Cancellation,
}

#[derive(NonFungibleData, ScryptoSbor, Clone)]
pub struct TxData {
    pub epoch: Decimal,
    pub app_handle: String,
    pub contract_handle: String,
    pub contract_address: ComponentAddress,

    pub from_handle: String,
    pub from_badge: ResourceAddress,
    pub to_handle: String,
    pub to_badge: ResourceAddress,

    pub resource_address: ResourceAddress,
    pub amount: Decimal,
    pub tx_type: TxType,
}
