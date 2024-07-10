use crate::badge_manager::badge_manager::BadgeManager;
use crate::marketplace::marketplace::Marketplace;
use crate::member::member::Member;
use crate::types::*;
use crate::vesting_schedule::VestingSchedule;
use scrypto::prelude::*;

#[blueprint]
mod job {
    enable_method_auth! {
        roles {
            admin => updatable_by: [];
        },
        methods {
            init => PUBLIC;
            invite => restrict_to: [admin];
            remove => restrict_to: [admin, SELF];
            leave => PUBLIC;
            join => PUBLIC;
            deposit => restrict_to: [admin];
            details => restrict_to: [admin];
            withdraw => PUBLIC;
            cancellation => restrict_to: [admin];
            list => restrict_to: [admin];
            data => PUBLIC;
            role => PUBLIC;
        }
    }

    struct Job {
        badge_manager: Owned<BadgeManager>,
        team_handle: String,
        contract_handle: String,
        contract_name: String,
        image: Url,
        category: String,
        details: KeyValueStore<String, String>,
        marketplaces: HashSet<ComponentAddress>,

        admin_badge: ResourceAddress,
        admin_handle: String,
        member_badges: HashMap<ResourceAddress, String>,
        signatures: HashSet<ResourceAddress>,
        funds: FungibleVault,

        vesting_schedule: VestingSchedule,
        reserved: FungibleVault,
        is_cancelled: bool,
        created_epoch: Decimal,
        list_epoch: Decimal,
    }

    impl Job {
        pub fn instantiate(
            dapp_address: ComponentAddress,
            member_address: Option<ComponentAddress>,
            admin_badge: ResourceAddress,
            admin_proof: NonFungibleProof,
            team_handle: String,
            contract_handle: String,
            contract_name: String,
            resource_address: ResourceAddress,
            start_epoch: i64,
            cliff_epoch: Option<i64>,
            end_epoch: i64,
            vest_interval: i64,
            is_check_join: bool,
            image: String,
            category: String,
            details: HashMap<String, String>,
        ) -> (Global<Job>, NonFungibleBucket) {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Job::blueprint_id());

            let admin_handle = Self::get_proof_id(&admin_badge, admin_proof);
            let badge_manager =
                BadgeManager::new(component_address, ContractKind::Job, contract_name.clone());
            let new_details = KeyValueStore::<String, String>::new();
            for (key, value) in details.iter() {
                new_details.insert(key.to_owned(), value.to_owned());
            }

            let vesting_schedule = VestingSchedule::new(
                start_epoch,
                cliff_epoch,
                end_epoch,
                vest_interval,
                dec!(0),
                is_check_join,
            );

            let component = Self {
                badge_manager,
                team_handle,
                contract_handle,
                contract_name,
                image: Url::of(image),
                category,
                details: new_details,
                marketplaces: HashSet::new(),

                admin_badge,
                admin_handle,
                member_badges: HashMap::new(),
                signatures: HashSet::new(),
                funds: FungibleVault::new(resource_address),

                vesting_schedule,
                reserved: FungibleVault::new(resource_address),
                is_cancelled: false,
                created_epoch: Decimal::from(VestingSchedule::get_curr_epoch()),
                list_epoch: dec!(0),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles!(
                admin => rule!(require(admin_badge));
            ))
            .metadata(metadata! {
                init {
                    "name" => "Diamond Pay: Job Contract", locked;
                    "description" => "Reward a member using a vesting schedule", locked;
                    "info_url" => Url::of(INFO_URL), locked;
                    "dapp_definition" => GlobalAddress::from(dapp_address), locked;
                }
            })
            .with_address(address_reservation)
            .globalize();

            let admin_bucket = component.init();

            if member_address.is_some() {
                let member = Global::<Member>::from(member_address.unwrap());
                member.add_job(component_address);
            }

            (component, admin_bucket)
        }

        pub fn init(&mut self) -> NonFungibleBucket {
            assert!(self.badge_manager.is_new(), "[Init Tx]: Already added");
            let contract_address = Runtime::global_address();
            let admin_bucket = self.badge_manager.create_admin_nft(
                self.admin_handle.clone(),
                BadgeData {
                    contract_address,
                    contract_kind: ContractKind::Job,
                    contract_role: ContractRole::Admin,
                    contract_handle: self.contract_handle.clone(),
                    team_handle: self.team_handle.clone(),
                },
            );

            // CREATE TXS
            self.create_tx(
                self.admin_handle.clone(),
                self.admin_badge,
                self.contract_handle.clone(),
                self.badge_manager.badge(),
                self.funds.amount(),
                TxType::Create,
            );

            admin_bucket.as_non_fungible()
        }

        pub fn invite(&mut self, member_badge: ResourceAddress, member_handle: String) {
            assert!(self.member_badges.is_empty(), "[Invite]: Already Added");
            let is_valid = ResourceManager::from(member_badge)
                .non_fungible_exists(&BadgeManager::nft_id(member_handle.clone()));
            assert!(is_valid, "[Invite]: Not valid");
            self.member_badges
                .insert(member_badge, member_handle.clone());

            // CREATE TXS
            self.create_tx(
                self.admin_handle.clone(),
                self.admin_badge,
                member_handle,
                member_badge,
                dec!(0),
                TxType::Invite,
            );
        }

        pub fn remove(&mut self, member_badge: ResourceAddress) {
            let is_signed = self.signatures.remove(&member_badge);
            assert!(!is_signed, "[Remove]: Already Signed");
            let handle = self.member_badges.remove(&member_badge).unwrap();
            // CREATE TXS
            self.create_tx(
                self.admin_handle.clone(),
                self.admin_badge,
                handle,
                member_badge,
                dec!(0),
                TxType::Remove,
            );
        }

        pub fn leave(&mut self, member_badge: ResourceAddress, proof: NonFungibleProof) {
            let member_handle = self.check_proof(&member_badge, proof);

            assert!(!self.is_cancelled, "[Leave]: Already Cancelled");
            self.set_reserved();
            assert!(self.reserved.amount() == dec!("0"), "[Leave]: Not zero");
            self.set_cancelled();

            // CREATE TXS
            self.create_tx(
                member_handle,
                member_badge,
                self.contract_handle.clone(),
                self.badge_manager.badge(),
                dec!(0),
                TxType::Leave,
            );
        }

        pub fn join(
            &mut self,
            member_badge: ResourceAddress,
            proof: NonFungibleProof,
        ) -> NonFungibleBucket {
            self.vesting_schedule.check_join();
            let member_handle = self.check_proof(&member_badge, proof);

            assert!(!self.signatures.contains(&member_badge), "[Join]: Signed");
            self.signatures.insert(member_badge);

            let contract_address = Runtime::global_address();
            let member_bucket = self.badge_manager.create_member_nft(
                member_handle.clone(),
                BadgeData {
                    contract_address,
                    contract_kind: ContractKind::Job,
                    contract_role: ContractRole::Member,
                    contract_handle: self.contract_handle.clone(),
                    team_handle: self.team_handle.clone(),
                },
            );

            // CREATE TXS
            self.create_tx(
                member_handle.clone(),
                member_badge,
                self.contract_handle.clone(),
                self.badge_manager.badge(),
                dec!(0),
                TxType::Join,
            );

            member_bucket
        }

        pub fn deposit(&mut self, funds: FungibleBucket) {
            assert!(!self.is_cancelled, "[Deposit]: Is Cancelled");
            Self::check_funds(&funds);

            // CREATE TXS
            self.create_tx(
                self.admin_handle.clone(),
                self.admin_badge,
                self.contract_handle.clone(),
                self.badge_manager.badge(),
                funds.amount(),
                TxType::Deposit,
            );

            self.vesting_schedule.amount = self.vesting_schedule.amount + funds.amount();
            self.funds.put(funds);
        }

        pub fn details(&mut self, image: String, details: HashMap<String, String>) {
            self.image = Url::of(image);
            for (key, value) in details.iter() {
                self.details.insert(key.to_owned(), value.to_owned());
            }

            // CREATE TXS
            self.create_tx(
                self.admin_handle.clone(),
                self.admin_badge,
                self.contract_handle.clone(),
                self.badge_manager.badge(),
                dec!(0),
                TxType::Details,
            );
        }

        pub fn withdraw(
            &mut self,
            member_badge: ResourceAddress,
            proof: NonFungibleProof,
        ) -> FungibleBucket {
            self.check_list();
            let member_handle = self.check_proof(&member_badge, proof);

            assert!(!self.signatures.is_empty(), "[Withdraw]: Not Signed");
            self.set_reserved();
            let withdraw_bucket = self.reserved.take_all();
            let amount = withdraw_bucket.amount();
            assert!(amount > dec!("0"), "[Withdraw]: Must not be zero");
            self.vesting_schedule.withdrawn = self.vesting_schedule.withdrawn + amount;

            // CREATE TXS
            self.create_tx(
                self.contract_handle.clone(),
                self.badge_manager.badge(),
                member_handle,
                member_badge,
                amount,
                TxType::Withdraw,
            );

            withdraw_bucket
        }

        pub fn cancellation(&mut self) -> FungibleBucket {
            self.check_list();

            if !self.signatures.is_empty() {
                self.set_reserved();
            }
            if !self.is_cancelled {
                self.set_cancelled();
            }
            let total = self.funds.take_all();

            // CREATE TXS
            self.create_tx(
                self.contract_handle.clone(),
                self.badge_manager.badge(),
                self.admin_handle.clone(),
                self.admin_badge,
                total.amount(),
                TxType::Cancellation,
            );

            total
        }

        pub fn list(&mut self, marketplace_address: ComponentAddress) {
            let marketplace = Global::<Marketplace>::from(marketplace_address);
            marketplace.check_contract(
                self.category.clone(),
                ContractKind::Job,
                self.vesting_schedule.amount,
                self.funds.resource_address(),
            );
            assert!(self.marketplaces.is_empty(), "[List]: Already added");
            self.marketplaces.insert(marketplace_address);

            self.list_epoch = Decimal::from(VestingSchedule::get_curr_epoch());

            // CREATE TXS
            self.create_tx(
                self.admin_handle.clone(),
                self.admin_badge,
                self.contract_handle.clone(),
                self.badge_manager.badge(),
                dec!(0),
                TxType::List,
            );
        }

        pub fn data(
            &self,
        ) -> (
            HashSet<ComponentAddress>,
            String,
            ResourceAddress,
            Decimal,
            ResourceAddress,
            bool,
            ComponentAddress,
        ) {
            (
                self.marketplaces.clone(),
                self.category.clone(),
                self.admin_badge,
                self.vesting_schedule.amount,
                self.funds.resource_address(),
                self.member_badges.is_empty() && !self.is_cancelled,
                Runtime::global_address(),
            )
        }

        pub fn role(&self, member_badge: ResourceAddress) -> ContractRole {
            let is_admin = self.admin_badge == member_badge;
            let is_member = self.signatures.contains(&member_badge);
            if is_admin {
                ContractRole::Admin
            } else if is_member {
                ContractRole::Member
            } else {
                ContractRole::Nonmember
            }
        }

        // Private Funcs

        fn set_reserved(&mut self) {
            if self.is_cancelled || self.vesting_schedule.amount == dec!(0) {
                return;
            }
            // Withdraw amount is the difference between the funds
            // in the vault right now and the unvested amount
            let unvested = self.vesting_schedule.get_unvested();
            let withdraw_amount = self.funds.amount() - unvested;

            // place unclaimed vested tokens in reserved bucket
            if withdraw_amount > dec!(0) {
                let withdraw_bucket = self.funds.take_advanced(
                    withdraw_amount,
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                );
                self.reserved.put(withdraw_bucket);
            }
        }

        fn set_cancelled(&mut self) {
            let cancel_epoch = VestingSchedule::get_curr_epoch();
            self.is_cancelled = true;
            self.vesting_schedule.cancel_epoch = Some(cancel_epoch);
        }

        fn check_list(&self) {
            assert!(
                Decimal::from(VestingSchedule::get_curr_epoch())
                    >= self.list_epoch + SEC_IN_DAY * LOCK_PERIOD,
                "[Check List]: Must be after listing period"
            );
        }

        fn create_tx(
            &self,
            from_handle: String,
            from_badge: ResourceAddress,
            to_handle: String,
            to_badge: ResourceAddress,
            amount: Decimal,
            tx_type: TxType,
        ) {
            let tx_data = TxData {
                epoch: Decimal::from(VestingSchedule::get_curr_epoch()),

                from_handle,
                from_badge,
                to_handle,
                to_badge,

                amount,
                tx_type,
            };
            self.badge_manager.create_tx(tx_data);
        }

        fn check_proof(&self, member_badge: &ResourceAddress, proof: NonFungibleProof) -> String {
            let handle = Self::get_proof_id(member_badge, proof);
            let saved_handle = self.member_badges.get(&member_badge).unwrap();
            assert!(&handle == saved_handle, "[Check Proof]: Not Equal");
            handle
        }

        fn get_proof_id(badge: &ResourceAddress, proof: NonFungibleProof) -> String {
            let result = proof.check(badge.clone());
            let string_id = match result.non_fungible_local_id() {
                NonFungibleLocalId::String(string_id) => string_id,
                _ => Runtime::panic(String::from("Invalid ID")),
            };
            string_id.value().to_owned()
        }

        fn check_funds(funds: &FungibleBucket) {
            let resource = ResourceManager::from_address(funds.resource_address()).resource_type();
            assert!(
                matches!(resource, ResourceType::Fungible { .. }),
                "[Check Funds]: Must be Fungible"
            );
            assert!(!funds.is_empty(), "[Check Funds]: Missing Funds");
        }
    }
}
