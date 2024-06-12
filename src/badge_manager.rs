use crate::contract_types::*;
use scrypto::prelude::*;

#[blueprint]
#[types(BadgeData, TxData)]
mod badge_manager {
    // dApp definition address used by ft_builder and nft_builder
    const DAPP_ACCOUNT: Global<Account> = global_component!(
        Account,
        "account_tdx_2_12893a32aeygqc4667dws2xfa30rr80lmd9z7lmu9x0fcxv2ckh460z"
    );

    struct BadgeManager {
        auth: FungibleVault,            // internal auth badge used for minting fts & nfts
        admin_manager: ResourceManager, // ResourceManager to mint admin nft with contract data
        member_manager: ResourceManager, // ResourceManager to mint member nfts with contract data
        tx_manager: ResourceManager,    // ResourceManager to mint transaction nfts
        tx_vault: NonFungibleVault,     // vault to store contract transaction nfts
        years: HashSet<Decimal>,        // which years transactions have taken place
        kind: String,                   // kind of contract: Project | Job
    }

    impl BadgeManager {
        /// Creates a new BadgeManager
        ///
        /// BadgeManagers are used for the following:
        /// 1. Contract Data Nfts - issued to the admin and members of the contract
        /// 2. Transaction Nfts - tracks all transactions executed and stores them
        ///
        /// # Arguments
        ///
        /// * `kind` - Project | Job; a string indicating the kind of contract
        /// * `name` - Name of the contract passed to the nft managers
        ///
        /// # Returns
        ///
        /// * `Owned<BadgeManager>` - The created BadgeManager
        ///
        pub fn new(kind: String, name: String) -> Owned<BadgeManager> {
            let auth_bucket = Self::ft_builder(
                "AUTH",
                &format!("Auth Badge"),
                &format!("Auth badge used for the contract"),
                &rule!(deny_all),
            )
            .mint_initial_supply(1);
            let auth_rule = rule!(require(auth_bucket.resource_address()));

            let admin_manager = Self::nft_builder::<BadgeData>(
                &format!("{kind} Admin: {name}"),
                &format!("Admin nft containing information on the contract"),
                &auth_rule,
            );

            let member_manager = Self::nft_builder::<BadgeData>(
                &format!("{kind} Member: {name}"),
                &format!("Member nft containing information on the contract"),
                &auth_rule,
            );

            let tx_manager = Self::nft_builder::<TxData>(
                &format!("{kind}Tx"),
                &format!("NFTs for tracking {kind} txs"),
                &auth_rule,
            );

            let component = Self {
                auth: FungibleVault::with_bucket(auth_bucket),
                admin_manager,
                member_manager,
                tx_manager,
                tx_vault: NonFungibleVault::new(tx_manager.address()),
                years: HashSet::new(),
                kind,
            }
            .instantiate();

            component
        }

        pub fn is_new(&self) -> bool {
            self.tx_vault.is_empty()
        }

        pub fn badge(&self) -> ResourceAddress {
            self.auth.resource_address()
        }

        pub fn create_admin_nft(
            &mut self,
            handle: String,
            nft_data: BadgeData,
        ) -> NonFungibleBucket {
            self.auth.authorize_with_amount(1, || {
                self.admin_manager
                    .mint_non_fungible(&Self::nft_id(handle), nft_data)
                    .as_non_fungible()
            })
        }

        pub fn create_member_nft(
            &mut self,
            handle: String,
            nft_data: BadgeData,
        ) -> NonFungibleBucket {
            self.auth.authorize_with_amount(1, || {
                self.member_manager
                    .mint_non_fungible(&Self::nft_id(handle), nft_data)
                    .as_non_fungible()
            })
        }

        pub fn create_tx(&mut self, tx_data: TxData) {
            let total = self.tx_manager.total_supply().unwrap() + 1;
            let bucket = self.auth.authorize_with_amount(1, || {
                self.tx_manager
                    .mint_non_fungible(&Self::nft_id(format!("{total}")), tx_data)
                    .as_non_fungible()
            });
            self.tx_vault.put(bucket);

            let year = Self::get_year();
            if !self.years.contains(&year) {
                self.years.insert(year);
            }
        }

        //
        //
        // Helper Functions -------------------------------------

        fn ft_builder(
            symbol: &str,
            name: &str,
            description: &str,
            access_rule: &AccessRule,
        ) -> InProgressResourceBuilder<FungibleResourceType> {
            ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata! {
                    init {
                      "symbol" => symbol, locked;
                      "name" => name, locked;
                      "description" => description, locked;
                      "tags" => ["badge"], locked;
                      "icon_url" => Url::of(ICON_URL), locked;
                      "info_url" => Url::of(INFO_URL), locked;
                      "dapp_definitions" => [Self::dapp_address()], locked;
                    }
                })
                .mint_roles(mint_roles! {
                    minter => access_rule.clone();
                    minter_updater => rule!(deny_all);
                })
        }

        fn nft_builder<D: BadgeManagerRegisteredType + NonFungibleData>(
            name: &str,
            description: &str,
            access_rule: &AccessRule,
        ) -> ResourceManager {
            ResourceBuilder::new_string_non_fungible_with_registered_type::<D>(OwnerRole::None)
                .metadata(metadata! {
                    init {
                      "name" => name, locked;
                      "description" => description, locked;
                      "tags" => ["badge"], locked;
                      "icon_url" => Url::of(ICON_URL), locked;
                      "info_url" => Url::of(INFO_URL), locked;
                      "dapp_definitions" => [Self::dapp_address()], locked;
                    }
                })
                .mint_roles(mint_roles! {
                    minter => access_rule.clone();
                    minter_updater => rule!(deny_all);
                })
                .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                    non_fungible_data_updater => access_rule.clone();
                    non_fungible_data_updater_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply()
        }

        pub fn nft_id(id: String) -> NonFungibleLocalId {
            let nft_str = StringNonFungibleLocalId::new(id).unwrap();
            NonFungibleLocalId::String(nft_str)
        }

        fn get_year() -> Decimal {
            let instant = Clock::current_time(TimePrecision::Second);
            let date = UtcDateTime::from_instant(&instant).unwrap();
            Decimal::from(date.year())
        }

        fn dapp_address() -> GlobalAddress {
            GlobalAddress::from(DAPP_ACCOUNT.address())
        }
    }
}
