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
            add_market => restrict_to: [admin];
            update_market => restrict_to: [admin];
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
        details: KeyValueStore<String, String>,
    }

    impl Marketplace {
        pub fn instantiate(
            admin_badge: ResourceAddress,
            name: String,
            dapp_address: ComponentAddress,
            markets: Vec<String>,
            resource_address: ResourceAddress,
        ) -> Global<Marketplace> {
            let projects = KeyValueStore::<String, Owned<MarketManager>>::new();
            let jobs = KeyValueStore::<String, Owned<MarketManager>>::new();

            for market in markets {
                let all_projects = MarketManager::new(
                    market.clone(),
                    ContractKind::Project,
                    dec!(2000),
                    resource_address,
                );
                projects.insert(market.clone(), all_projects);

                let all_jobs = MarketManager::new(
                    market.clone(),
                    ContractKind::Job,
                    dec!(2000),
                    resource_address,
                );
                jobs.insert(market, all_jobs);
            }

            let component = Self {
                admin_badge,
                name,
                projects,
                jobs,
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

        pub fn add_market(
            &mut self,
            name: String,
            is_project: bool,
            minimum: Decimal,
            resource_address: ResourceAddress,
        ) {
            let kind = match is_project {
                true => ContractKind::Project,
                false => ContractKind::Job,
            };
            let market = MarketManager::new(name.clone(), kind.clone(), minimum, resource_address);
            match kind {
                ContractKind::Project => self.projects.insert(name, market),
                ContractKind::Job => self.jobs.insert(name, market),
            };
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
        ) {
            let project = Global::<ProjectContract>::from(project_address);
            let (
                marketplace_address,
                admin_badge,
                contract_amount,
                contract_resource,
                is_joinable,
                contract_address,
            ) = project.data();
            proof.check(admin_badge);
            assert!(
                marketplace_address == Runtime::global_address(),
                "[Add Project]: Marketplace address not the same"
            );
            assert!(is_joinable, "[Add Project]: Not joinable");
            let market = self.projects.get(&name).unwrap();
            market.check_contract(contract_address, contract_amount, contract_resource);
            market.list(contract_address);
        }

        pub fn add_job(
            &mut self,
            name: String,
            job_address: ComponentAddress,
            proof: NonFungibleProof,
        ) {
            let job = Global::<JobContract>::from(job_address);
            let (
                marketplace_address,
                admin_badge,
                contract_amount,
                contract_resource,
                is_joinable,
                contract_address,
            ) = job.data();
            proof.check(admin_badge);
            assert!(
                marketplace_address == Runtime::global_address(),
                "[Add Job]: Marketplace address not the same"
            );
            assert!(is_joinable, "[Add Job]: Not joinable");
            let market = self.jobs.get(&name).unwrap();
            market.check_contract(contract_address, contract_amount, contract_resource);
            market.list(contract_address);
        }
    }
}
