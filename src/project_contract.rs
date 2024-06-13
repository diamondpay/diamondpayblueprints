use crate::badge_manager::badge_manager::BadgeManager;
use crate::contract_types::*;
use scrypto::prelude::*;

#[blueprint]
mod project_contract {
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
            update => restrict_to: [admin];
            reward => restrict_to: [admin];
            complete => restrict_to: [admin];
            withdraw => PUBLIC;
            cancellation => restrict_to: [admin];
        }
    }

    struct ProjectContract {
        badge_manager: Owned<BadgeManager>,
        app_handle: String,
        contract_handle: String,
        contract_name: String,

        admin_badge: ResourceAddress,
        admin_handle: String,
        member_badges: HashMap<ResourceAddress, String>,
        removed: HashMap<ResourceAddress, String>,
        signatures: HashSet<ResourceAddress>,
        funds: FungibleVault,

        objectives: HashMap<Decimal, HashMap<ResourceAddress, Decimal>>,
        completed: HashMap<Decimal, HashMap<ResourceAddress, Decimal>>,
        reserved: HashMap<ResourceAddress, FungibleVault>,
        is_cancelled: bool,
        cancelled_epoch: Decimal,
        created_epoch: Decimal,
    }

    impl ProjectContract {
        pub fn instantiate(
            app_handle: String,
            contract_handle: String,
            contract_name: String,
            admin_badge: ResourceAddress,
            admin_proof: NonFungibleProof,
            resource_address: ResourceAddress,
        ) -> (Global<ProjectContract>, NonFungibleBucket) {
            let admin_handle = Self::get_proof_id(&admin_badge, admin_proof);
            let badge_manager = BadgeManager::new("Project".to_string(), contract_name.clone());

            let component = Self {
                badge_manager,
                app_handle,
                contract_handle,
                contract_name,

                admin_badge,
                admin_handle,
                member_badges: HashMap::new(),
                removed: HashMap::new(),
                signatures: HashSet::new(),
                funds: FungibleVault::new(resource_address),

                objectives: HashMap::new(),
                completed: HashMap::new(),
                reserved: HashMap::new(),
                is_cancelled: false,
                cancelled_epoch: dec!(0),
                created_epoch: Self::get_curr_epoch(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles!(
                admin => rule!(require(admin_badge));
            ))
            .metadata(metadata! {
                init {
                    "name" => "Diamond Pay: Project Contract", locked;
                    "description" => "Reward members using objectives", locked;
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
            assert!(self.member_badges.len() <= 30, "[Invite]: Too many members");
            assert!(
                !self.member_badges.contains_key(&member_badge),
                "[Add Member]: Already Added"
            );
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
            for (_, members) in self.objectives.iter_mut() {
                members.remove(&member_badge);
            }

            let handle = self.member_badges.remove(&member_badge).unwrap();
            if self.signatures.contains(&member_badge) {
                self.removed.insert(member_badge, handle.clone());
            }

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
            self.check_proof(&member_badge, proof);
            self.remove(member_badge);
        }

        pub fn join(
            &mut self,
            member_badge: ResourceAddress,
            proof: NonFungibleProof,
        ) -> NonFungibleBucket {
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

            self.funds.put(funds);
        }

        pub fn update(&mut self, objectives: HashMap<Decimal, HashMap<ResourceAddress, Decimal>>) {
            let total_objs = objectives.len() + self.completed.len();
            assert!(total_objs <= 30, "[Update]: Too many objectives");

            let mut total = dec!("0");
            for (obj_num, members) in objectives.iter() {
                assert!(!members.is_empty(), "[Update]: Empty Members");
                assert!(members.len() <= 10, "[Update]: Too many members");
                for (member, amount) in members.iter() {
                    if self.completed.contains_key(obj_num) {
                        let com_dis = self.completed.get(obj_num).unwrap();
                        assert!(!com_dis.contains_key(member), "[Update]: Completed");
                    }
                    assert!(
                        self.member_badges.contains_key(member),
                        "[Update]: No Member"
                    );
                    assert!(amount > &dec!("0"), "[Update]: No Amount");
                    total = total + amount.clone();
                }
            }
            assert!(total == self.funds.amount(), "[Update]: Invalid Sum");
            self.objectives = objectives;

            // CREATE TXS
            self.create_tx(
                self.admin_handle.clone(),
                self.admin_badge,
                self.contract_handle.clone(),
                self.badge_manager.badge(),
                dec!(0),
                TxType::Update,
            );
        }

        pub fn reward(&mut self, obj_number: Decimal, member: ResourceAddress) {
            assert!(self.signatures.contains(&member), "[Reward]: No signature");
            // remove mutable distributions from objectives
            let mut distributions = self.objectives.remove(&obj_number).unwrap();
            let amount: Decimal = distributions.get(&member).unwrap().clone();

            let pay_bucket = self
                .funds
                .take_advanced(amount, WithdrawStrategy::Rounded(RoundingMode::ToZero));
            assert!(!pay_bucket.is_empty(), "[Reward]: No funds");

            // add distribution to completed
            if self.completed.contains_key(&obj_number) {
                let com_dis = self.completed.get_mut(&obj_number).unwrap();
                assert!(!com_dis.contains_key(&member), "[Reward]: Invalid");
                com_dis.insert(member.clone(), amount.clone());
            } else {
                let mut com_dis = HashMap::<ResourceAddress, Decimal>::new();
                com_dis.insert(member.clone(), amount.clone());
                self.completed.insert(obj_number.clone(), com_dis.clone());
            }
            // remove distribution from objective distributions
            // add updated distribution back into objectives
            distributions.remove(&member);
            if !distributions.is_empty() {
                self.objectives.insert(obj_number, distributions);
            }

            // CREATE TXS
            let handle = self.member_badges.get(&member).unwrap().to_string();
            self.create_tx(
                self.contract_handle.clone(),
                self.badge_manager.badge(),
                handle,
                member,
                pay_bucket.amount(),
                TxType::Reward,
            );

            // Deposit into Reserved Vaults
            if !self.reserved.contains_key(&member) {
                self.reserved
                    .insert(member, FungibleVault::with_bucket(pay_bucket));
            } else {
                let vault = self.reserved.get_mut(&member).unwrap();
                vault.put(pay_bucket);
            }
        }

        pub fn complete(&mut self, obj_number: Decimal) {
            let members = self.objectives.get(&obj_number).unwrap().clone();
            for (member, _) in members.iter() {
                self.reward(obj_number, member.clone());
            }
        }

        pub fn withdraw(
            &mut self,
            member_badge: ResourceAddress,
            proof: NonFungibleProof,
        ) -> FungibleBucket {
            let member_handle = self.check_proof(&member_badge, proof);
            let vault = self.reserved.get_mut(&member_badge).unwrap();
            let bucket = vault.take_all();
            assert!(!bucket.is_empty(), "[Withdraw]: Is empty");

            // CREATE TXS
            self.create_tx(
                self.contract_handle.clone(),
                self.badge_manager.badge(),
                member_handle,
                member_badge,
                bucket.amount(),
                TxType::Withdraw,
            );

            bucket
        }

        pub fn cancellation(&mut self) -> FungibleBucket {
            self.objectives = HashMap::new();
            self.is_cancelled = true;
            self.cancelled_epoch = Self::get_curr_epoch();
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
                epoch: Self::get_curr_epoch(),
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

        fn get_curr_epoch() -> Decimal {
            let epoch = Clock::current_time(TimePrecision::Second).seconds_since_unix_epoch;
            Decimal::from(epoch)
        }

        fn dapp_address() -> GlobalAddress {
            GlobalAddress::from(DAPP_ACCOUNT.address())
        }
    }
}
