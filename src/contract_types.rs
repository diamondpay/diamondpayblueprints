use scrypto::prelude::*;

pub const INFO_URL: &str = "https://github.com/diamondpay";
pub const ICON_URL: &str = "https://avatars.githubusercontent.com/u/162780104";
pub const SEC_IN_DAY: i64 = 60i64 * 60i64 * 24i64;
pub const MAX_MEMBERS: usize = 10;
pub const MAX_OBJS: usize = 30;
pub const MAX_MARKETPLACES: usize = 3;
pub const MAX_MARKETS: usize = 3;
pub const LOCK_PERIOD: i64 = 5;
pub const MEMBER_ADDRESS: &str = "member_address";

#[derive(ScryptoSbor, PartialEq, Clone)]
pub enum ContractKind {
    Project,
    Job,
}
#[derive(ScryptoSbor, PartialEq)]
pub enum ContractRole {
    Admin,
    Member,
}

#[derive(NonFungibleData, ScryptoSbor)]
pub struct BadgeData {
    pub contract_address: ComponentAddress,
    pub contract_kind: ContractKind,
    pub contract_role: ContractRole,
    pub contract_handle: String,
    pub team_handle: String,
}

#[derive(NonFungibleData, ScryptoSbor)]
pub struct MemberData {
    pub member_address: ComponentAddress,
}

#[derive(ScryptoSbor)]
pub struct TeamData {
    // name, icon_url, team_handle, subtitle, description
    // social_urls, link_urls, image_urls, video_ids
    pub details: HashMap<String, String>,
    pub team_badge: Option<ResourceAddress>,
}

#[derive(ScryptoSbor, Clone)]
pub enum TxType {
    Create,
    Details,
    Deposit,
    List,
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
    pub from_handle: String,
    pub from_badge: ResourceAddress,
    pub to_handle: String,
    pub to_badge: ResourceAddress,
    pub amount: Decimal,
    pub tx_type: TxType,
}
