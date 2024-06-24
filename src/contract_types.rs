use scrypto::prelude::*;

pub const INFO_URL: &str = "https://github.com/diamondpay";
pub const ICON_URL: &str = "https://avatars.githubusercontent.com/u/162780104";
pub const SEC_IN_DAY: i64 = 60i64 * 60i64 * 24i64;
pub const PROJECT: &str = "Project";
pub const JOB: &str = "Job";
pub const MAX_MEMBERS: usize = 10;
pub const MAX_OBJS: usize = 30;
pub const MAX_MARKETS: usize = 3;

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
    pub app_handle: String,
}

#[derive(NonFungibleData, ScryptoSbor)]
pub struct MemberData {
    pub member_address: ComponentAddress,
}

#[derive(ScryptoSbor)]
pub struct AppData {
    // handle, name, subtitle, description, video_ids
    pub data: HashMap<String, String>,
    // icon_url, social_urls, link_urls, image_urls
    pub urls: HashMap<String, Vec<Url>>,
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
