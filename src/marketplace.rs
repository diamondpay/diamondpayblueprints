use crate::contract_types::*;
use crate::job_contract::job_contract::JobContract;
use crate::market_manager::market_manager::MarketManager;
use crate::project_contract::project_contract::ProjectContract;
use scrypto::prelude::*;

#[blueprint]
mod marketplace {
    enable_method_auth! {
        roles {
            admin => updatable_by: [];
        },
        methods {
            update => restrict_to: [admin];
            add_markets => restrict_to: [admin];
            update_market => restrict_to: [admin];
            remove_contract => restrict_to: [admin];
            deposit => restrict_to: [admin, SELF];
            withdraw => restrict_to: [admin, SELF];
            check_contract => PUBLIC;
            add_project => PUBLIC;
            add_job => PUBLIC;
        }
    }

    struct Marketplace {
        admin_badge: ResourceAddress,
        name: String,
        projects: KeyValueStore<String, Owned<MarketManager>>,
        jobs: KeyValueStore<String, Owned<MarketManager>>,
        resources: KeyValueStore<ResourceAddress, Vault>,
        details: KeyValueStore<String, String>,
    }

    impl Marketplace {
        pub fn instantiate(
            admin_badge: ResourceAddress,
            name: String,
            dapp_address: ComponentAddress,
            markets: Vec<String>,
            minimum: Decimal,
            fee: Decimal,
            resource_address: ResourceAddress,
        ) -> Global<Marketplace> {
            let projects = KeyValueStore::<String, Owned<MarketManager>>::new();
            let jobs = KeyValueStore::<String, Owned<MarketManager>>::new();

            for market in markets {
                let all_projects = MarketManager::new(
                    market.clone(),
                    ContractKind::Project,
                    minimum,
                    fee,
                    resource_address,
                );
                projects.insert(market.clone(), all_projects);

                let all_jobs = MarketManager::new(
                    market.clone(),
                    ContractKind::Job,
                    minimum,
                    fee,
                    resource_address,
                );
                jobs.insert(market, all_jobs);
            }

            let component = Self {
                admin_badge,
                name,
                projects,
                jobs,
                resources: KeyValueStore::new(),
                details: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles!(
                admin => rule!(require(admin_badge));
            ))
            .metadata(metadata! {
                init {
                    "name" => "Diamond Pay: Marketplace", locked;
                    "description" => "Creates a marketplace for Diamond Pay Contracts", locked;
                    "info_url" => Url::of(INFO_URL), locked;
                    "dapp_definition" => GlobalAddress::from(dapp_address), locked;
                }
            })
            .globalize();

            component
        }

        pub fn update(&mut self, name: String, details: HashMap<String, String>) {
            self.name = name;
            for (key, value) in details.iter() {
                self.details.insert(key.to_owned(), value.to_owned());
            }
        }

        pub fn add_markets(
            &mut self,
            names: Vec<String>,
            minimum: Decimal,
            fee: Decimal,
            resource_address: ResourceAddress,
        ) {
            for name in names {
                let market = MarketManager::new(
                    name.clone(),
                    ContractKind::Project,
                    minimum,
                    fee,
                    resource_address,
                );
                self.projects.insert(name.clone(), market);
                let market = MarketManager::new(
                    name.clone(),
                    ContractKind::Job,
                    minimum,
                    fee,
                    resource_address,
                );
                self.jobs.insert(name, market);
            }
        }

        pub fn update_market(
            &mut self,
            name: String,
            is_project: bool,
            minimum: Decimal,
            details: HashMap<String, String>,
        ) {
            if is_project {
                self.projects
                    .get(&name)
                    .unwrap()
                    .update(name, minimum, details);
            } else {
                self.jobs.get(&name).unwrap().update(name, minimum, details);
            }
        }

        pub fn remove_contract(
            &mut self,
            component_address: ComponentAddress,
            name: String,
            is_project: bool,
        ) {
            if is_project {
                self.projects.get(&name).unwrap().remove(component_address);
            } else {
                self.jobs.get(&name).unwrap().remove(component_address);
            }
        }

        pub fn deposit(&mut self, bucket: Bucket) {
            let resource_address = bucket.resource_address();
            let has_resource = self.resources.get(&resource_address).is_some();
            if has_resource {
                self.resources
                    .get_mut(&resource_address)
                    .unwrap()
                    .put(bucket);
            } else {
                self.resources
                    .insert(resource_address, Vault::with_bucket(bucket));
            }
        }

        pub fn withdraw(&mut self, resource_address: ResourceAddress) -> Bucket {
            self.resources
                .get_mut(&resource_address)
                .unwrap()
                .take_all()
        }

        pub fn check_contract(
            &mut self,
            name: String,
            kind: ContractKind,
            contract_address: ComponentAddress,
            contract_amount: Decimal,
            contract_resource: ResourceAddress,
        ) {
            if kind == ContractKind::Project {
                self.projects.get(&name).unwrap().check_contract(
                    contract_address,
                    contract_amount,
                    contract_resource,
                );
            } else {
                self.jobs.get(&name).unwrap().check_contract(
                    contract_address,
                    contract_amount,
                    contract_resource,
                );
            }
        }

        pub fn add_project(
            &mut self,
            name: String,
            project_address: ComponentAddress,
            proof: NonFungibleProof,
            fee_bucket: FungibleBucket,
        ) {
            let project = Global::<ProjectContract>::from(project_address);
            let (
                marketplaces,
                admin_badge,
                contract_amount,
                contract_resource,
                is_joinable,
                contract_address,
            ) = project.data();
            proof.check(admin_badge);
            assert!(
                marketplaces.contains_key(&Runtime::global_address()),
                "[Add Project]: Marketplace addresses must be the same"
            );
            assert!(is_joinable, "[Add Project]: Not joinable");
            let market = self.projects.get(&name).unwrap();

            let market_fee =
                market.check_contract(contract_address, contract_amount, contract_resource);
            assert!(
                fee_bucket.amount() == market_fee,
                "[Add Project]: Missing fee"
            );
            assert!(
                fee_bucket.resource_address() == XRD,
                "[Add Project]: Fee must be XRD"
            );
            market.list(contract_address);
            drop(market);
            self.deposit(Bucket::from(fee_bucket));
        }

        pub fn add_job(
            &mut self,
            name: String,
            job_address: ComponentAddress,
            proof: NonFungibleProof,
            fee_bucket: FungibleBucket,
        ) {
            let job = Global::<JobContract>::from(job_address);
            let (
                marketplaces,
                admin_badge,
                contract_amount,
                contract_resource,
                is_joinable,
                contract_address,
            ) = job.data();
            proof.check(admin_badge);
            assert!(
                marketplaces.contains_key(&Runtime::global_address()),
                "[Add Job]: Marketplace addresses must be the same"
            );
            assert!(is_joinable, "[Add Job]: Not joinable");
            let market = self.jobs.get(&name).unwrap();

            let market_fee =
                market.check_contract(contract_address, contract_amount, contract_resource);
            assert!(fee_bucket.amount() == market_fee, "[Add Job]: Missing fee");
            assert!(
                fee_bucket.resource_address() == XRD,
                "[Add job]: Fee must be XRD"
            );
            market.list(contract_address);
            drop(market);
            self.deposit(Bucket::from(fee_bucket));
        }
    }
}
