use crate::badge_manager::badge_manager::BadgeManager;
use crate::contract_types::*;
use crate::vesting_schedule::VestingSchedule;
use scrypto::prelude::*;

#[blueprint]
mod job_contract {
    const DAPP_ACCOUNT: Global<Account> = global_component!(
        Account,
        "account_tdx_2_12893a32aeygqc4667dws2xfa30rr80lmd9z7lmu9x0fcxv2ckh460z"
    );

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
            withdraw => PUBLIC;
            cancellation => restrict_to: [admin];
        }
    }

    struct JobContract {
        badge_manager: Owned<BadgeManager>,
        app_handle: String,
        contract_handle: String,
        contract_name: String,

        admin_badge: ResourceAddress,
        admin_handle: String,
        member_badges: HashMap<ResourceAddress, String>,
        signatures: HashSet<ResourceAddress>,
        funds: FungibleVault,

        vesting_schedule: VestingSchedule,
        reserved: FungibleVault,
        is_cancelled: bool,
        created_epoch: Decimal,
    }

    impl JobContract {
        pub fn instantiate(
            app_handle: String,
            contract_handle: String,
            contract_name: String,
            admin_badge: ResourceAddress,
            admin_proof: NonFungibleProof,
            resource_address: ResourceAddress,
            start_epoch: i64,
            cliff_epoch: Option<i64>,
            end_epoch: i64,
            vest_interval: i64,
        ) -> (Global<JobContract>, NonFungibleBucket) {
            let admin_handle = Self::get_proof_id(&admin_badge, admin_proof);
            let badge_manager = BadgeManager::new("Job".to_string(), contract_name.clone());

            let vesting_schedule =
                VestingSchedule::new(start_epoch, cliff_epoch, end_epoch, vest_interval, dec!(0));

            let component = Self {
                badge_manager,
                app_handle,
                contract_handle,
                contract_name,
                admin_badge,
                admin_handle,
                member_badges: HashMap::new(),
                signatures: HashSet::new(),
                funds: FungibleVault::new(resource_address),
                vesting_schedule,
                reserved: FungibleVault::new(resource_address),
                is_cancelled: false,
                created_epoch: Decimal::from(VestingSchedule::get_curr_epoch()),
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
                    "dapp_definition" => Self::dapp_address(), locked;
                }
            })
            .globalize();

            let admin_bucket = component.init();

            (component, admin_bucket)
        }

        pub fn init(&mut self) -> NonFungibleBucket {
            assert!(self.badge_manager.is_new(), "[Init Tx]: Already added");
            let contract_address = Runtime::global_address();
            let admin_bucket = self.badge_manager.create_admin_nft(
                self.admin_handle.clone(),
                BadgeData {
                    contract_address,
                    contract_handle: self.contract_handle.clone(),
                    app_handle: self.app_handle.clone(),
                    member_handle: self.admin_handle.clone(),
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
                    contract_handle: self.contract_handle.clone(),
                    app_handle: self.app_handle.clone(),
                    member_handle: member_handle.clone(),
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

        pub fn withdraw(
            &mut self,
            member_badge: ResourceAddress,
            proof: NonFungibleProof,
        ) -> FungibleBucket {
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

        // // Private Funcs

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

        fn check_proof(&self, member_badge: &ResourceAddress, proof: NonFungibleProof) -> String {
            let handle = Self::get_proof_id(member_badge, proof);
            let saved_handle = self.member_badges.get(&member_badge).unwrap();
            assert!(&handle == saved_handle, "[Check Proof]: Not Equal");
            handle
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
                app_handle: self.app_handle.clone(),
                contract_handle: self.contract_handle.clone(),
                contract_address: Runtime::global_address(),

                from_handle,
                from_badge,
                to_handle,
                to_badge,

                resource_address: self.funds.resource_address(),
                amount,
                tx_type,
            };
            self.badge_manager.create_tx(tx_data);
        }

        fn get_proof_id(badge: &ResourceAddress, proof: NonFungibleProof) -> String {
            let result = proof.check(badge.clone());
            let string_id = match result.non_fungible_local_id() {
                NonFungibleLocalId::String(string_id) => string_id,
                _ => Runtime::panic(String::from("Invalid ID")),
            };
            string_id.value().to_string()
        }

        fn check_funds(funds: &FungibleBucket) {
            let resource = ResourceManager::from_address(funds.resource_address()).resource_type();
            assert!(
                matches!(resource, ResourceType::Fungible { .. }),
                "[Check Funds]: Must be Fungible"
            );
            assert!(!funds.is_empty(), "[Check Funds]: Missing Funds");
        }

        fn dapp_address() -> GlobalAddress {
            GlobalAddress::from(DAPP_ACCOUNT.address())
        }
    }
}
