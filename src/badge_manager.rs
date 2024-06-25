use crate::contract_types::*;
use scrypto::prelude::*;

#[blueprint]
#[types(BadgeData)]
mod badge_manager {
    struct BadgeManager {
        admin_manager: ResourceManager, // ResourceManager to mint admin nft with contract data
        member_manager: ResourceManager, // ResourceManager to mint member nfts with contract data
        txs: KeyValueStore<String, TxData>,
        txs_total: Decimal,
        years: HashSet<Decimal>, // which years transactions have taken place
        kind: ContractKind,      // kind of contract: Project | Job
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
        /// * `component_address` - Address of component that has auth permission
        /// * `kind` - Project | Job; a string indicating the kind of contract
        /// * `name` - Name of the contract passed to the nft managers
        ///
        /// # Returns
        ///
        /// * `Owned<BadgeManager>` - The created BadgeManager
        ///
        pub fn new(
            component_address: ComponentAddress,
            kind: ContractKind,
            name: String,
        ) -> Owned<BadgeManager> {
            let auth_rule = rule!(require(global_caller(component_address)));
            let kind_str = match kind {
                ContractKind::Project => "Project",
                ContractKind::Job => "Job",
            };

            let admin_manager = Self::nft_builder::<BadgeData>(
                &format!("{kind_str} Admin: {name}"),
                "Admin nft containing information on the contract",
                &auth_rule,
            );

            let member_manager = Self::nft_builder::<BadgeData>(
                &format!("{kind_str} Member: {name}"),
                "Member nft containing information on the contract",
                &auth_rule,
            );

            let component = Self {
                admin_manager,
                member_manager,
                txs: KeyValueStore::new(),
                txs_total: dec!(0),
                years: HashSet::new(),
                kind,
            }
            .instantiate();

            component
        }

        pub fn is_new(&self) -> bool {
            self.txs_total == dec!(0)
        }

        pub fn badge(&self) -> ResourceAddress {
            self.admin_manager.address()
        }

        pub fn create_admin_nft(
            &mut self,
            handle: String,
            nft_data: BadgeData,
        ) -> NonFungibleBucket {
            self.admin_manager
                .mint_non_fungible(&Self::nft_id(handle), nft_data)
                .as_non_fungible()
        }

        pub fn create_member_nft(
            &mut self,
            handle: String,
            nft_data: BadgeData,
        ) -> NonFungibleBucket {
            self.member_manager
                .mint_non_fungible(&Self::nft_id(handle), nft_data)
                .as_non_fungible()
        }

        pub fn create_tx(&mut self, tx_data: TxData) {
            let new_total = self.txs_total + 1;
            self.txs_total = new_total;
            self.txs.insert(format!("{new_total}"), tx_data);

            let year = Self::get_year();
            if !self.years.contains(&year) {
                self.years.insert(year);
            }
        }

        //
        //
        // Helper Functions -------------------------------------

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
                    }
                })
                .mint_roles(mint_roles! {
                    minter => access_rule.clone();
                    minter_updater => rule!(deny_all);
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
    }
}
