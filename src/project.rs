use crate::badge_manager::badge_manager::BadgeManager;
use crate::marketplace::marketplace::Marketplace;
use crate::member::member::Member;
use crate::types::*;
use scrypto::prelude::*;

#[blueprint]
mod project {
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
            details => restrict_to: [admin];
            reward => restrict_to: [admin];
            withdraw => PUBLIC;
            cancellation => restrict_to: [admin];
            list => restrict_to: [admin];
            data => PUBLIC;
            role => PUBLIC;
        }
    }

    struct Project {
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
        removed: HashMap<ResourceAddress, String>,
        signatures: HashSet<ResourceAddress>,
        funds: FungibleVault,
        resource_address: ResourceAddress,

        start_epoch: i64,
        end_epoch: i64,
        amount: Decimal,
        rewarded: Decimal,
        withdrawn: Decimal,

        objectives: HashMap<Decimal, HashMap<ResourceAddress, Decimal>>,
        completed: HashMap<Decimal, HashMap<ResourceAddress, Decimal>>,
        reserved: HashMap<ResourceAddress, FungibleVault>,
        is_joinable: bool,
        is_cancelled: bool,
        cancelled_epoch: Decimal,
        created_epoch: Decimal,
        list_epoch: Decimal,
    }

    impl Project {
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
            end_epoch: i64,
            image: String,
            category: String,
            details: HashMap<String, String>,
        ) -> (Global<Project>, NonFungibleBucket) {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Project::blueprint_id());

            let admin_handle = Self::get_proof_id(&admin_badge, admin_proof);
            let badge_manager = BadgeManager::new(
                component_address,
                ContractKind::Project,
                contract_name.clone(),
            );
            assert!(end_epoch >= start_epoch, "[Instantiate]: Invalid Dates");
            let new_details = KeyValueStore::<String, String>::new();
            for (key, value) in details.iter() {
                new_details.insert(key.to_owned(), value.to_owned());
            }

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
                removed: HashMap::new(),
                signatures: HashSet::new(),
                funds: FungibleVault::new(resource_address),
                resource_address,

                start_epoch,
                end_epoch,
                amount: dec!(0),
                rewarded: dec!(0),
                withdrawn: dec!(0),

                objectives: HashMap::new(),
                completed: HashMap::new(),
                reserved: HashMap::new(),
                is_joinable: true,
                is_cancelled: false,
                cancelled_epoch: dec!(0),
                created_epoch: Self::get_curr_epoch(),
                list_epoch: dec!(0),
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
                    "dapp_definition" => GlobalAddress::from(dapp_address), locked;
                }
            })
            .with_address(address_reservation)
            .globalize();

            let admin_bucket = component.init();

            if member_address.is_some() {
                let member = Global::<Member>::from(member_address.unwrap());
                member.add_project(component_address);
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
                    contract_kind: ContractKind::Project,
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
            assert!(
                self.member_badges.len() <= MAX_MEMBERS,
                "[Invite]: Too many members"
            );
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
                    contract_kind: ContractKind::Project,
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

            self.amount = self.amount + funds.amount();
            self.funds.put(funds);
        }

        pub fn update(&mut self, objectives: HashMap<Decimal, HashMap<ResourceAddress, Decimal>>) {
            let total_objs = objectives.len() + self.completed.len();
            assert!(total_objs <= MAX_OBJS, "[Update]: Too many objectives");

            let mut total = dec!("0");
            for (obj_num, members) in objectives.iter() {
                assert!(!members.is_empty(), "[Update]: Empty Members");
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

        pub fn details(
            &mut self,
            start_epoch: i64,
            end_epoch: i64,
            image: String,
            details: HashMap<String, String>,
            is_joinable: bool,
        ) {
            assert!(end_epoch >= start_epoch, "[Instantiate]: Invalid Dates");
            self.start_epoch = start_epoch;
            self.end_epoch = end_epoch;
            self.image = Url::of(image);
            for (key, value) in details.iter() {
                self.details.insert(key.to_owned(), value.to_owned());
            }
            self.is_joinable = is_joinable;

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

        pub fn reward(&mut self, obj_number: Decimal) {
            let members = self.objectives.remove(&obj_number).unwrap();
            for (member, amount) in members.iter() {
                assert!(self.signatures.contains(member), "[Reward]: No signature");
                let pay_bucket = self.funds.take_advanced(
                    amount.clone(),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                );
                assert!(!pay_bucket.is_empty(), "[Reward]: No funds");

                let new_amount = pay_bucket.amount();
                self.rewarded = self.rewarded + new_amount;
                // Deposit into Reserved Vaults
                if !self.reserved.contains_key(&member) {
                    self.reserved
                        .insert(member.clone(), FungibleVault::with_bucket(pay_bucket));
                } else {
                    let vault = self.reserved.get_mut(&member).unwrap();
                    vault.put(pay_bucket);
                }
                // CREATE TXS
                let handle = self.member_badges.get(&member).unwrap().to_owned();
                self.create_tx(
                    self.contract_handle.clone(),
                    self.badge_manager.badge(),
                    handle,
                    member.clone(),
                    new_amount,
                    TxType::Reward,
                );
            }
            assert!(
                !self.completed.contains_key(&obj_number),
                "[Reward]: Already completed"
            );
            self.completed.insert(obj_number, members);
        }

        pub fn withdraw(
            &mut self,
            member_badge: ResourceAddress,
            proof: NonFungibleProof,
        ) -> FungibleBucket {
            self.check_list();
            let member_handle = self.check_proof(&member_badge, proof);
            let vault = self.reserved.get_mut(&member_badge).unwrap();
            let bucket = vault.take_all();
            assert!(!bucket.is_empty(), "[Withdraw]: Is empty");
            let withdrawn = bucket.amount();
            self.withdrawn = self.withdrawn + withdrawn;

            // CREATE TXS
            self.create_tx(
                self.contract_handle.clone(),
                self.badge_manager.badge(),
                member_handle,
                member_badge,
                withdrawn,
                TxType::Withdraw,
            );

            bucket
        }

        pub fn cancellation(&mut self) -> FungibleBucket {
            self.check_list();
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

        pub fn list(&mut self, marketplace_address: ComponentAddress) {
            let marketplace = Global::<Marketplace>::from(marketplace_address);
            marketplace.check_contract(
                self.category.clone(),
                ContractKind::Project,
                self.amount,
                self.funds.resource_address(),
            );
            assert!(self.marketplaces.is_empty(), "[List]: Already added");
            self.marketplaces.insert(marketplace_address);

            self.list_epoch = Self::get_curr_epoch();

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
                self.amount - self.rewarded,
                self.funds.resource_address(),
                self.is_joinable && !self.is_cancelled,
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

        fn check_list(&self) {
            assert!(
                Self::get_curr_epoch() >= self.list_epoch + SEC_IN_DAY * LOCK_PERIOD,
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
                epoch: Self::get_curr_epoch(),
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

        fn get_curr_epoch() -> Decimal {
            let epoch = Clock::current_time(TimePrecision::Second).seconds_since_unix_epoch;
            Decimal::from(epoch)
        }
    }
}
