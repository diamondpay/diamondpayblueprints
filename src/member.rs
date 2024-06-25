use crate::contract_types::*;
use crate::job_contract::job_contract::JobContract;
use crate::project_contract::project_contract::ProjectContract;
use scrypto::prelude::*;

#[blueprint]
#[types(AppData, MemberData)]
mod member {
    enable_method_auth! {
        roles {
            admin => updatable_by: [];
        },
        methods {
            add_project => PUBLIC;
            add_job => PUBLIC;
            remove_contract => restrict_to: [admin];
            deposit_resource => restrict_to: [admin];
            withdraw_resource => restrict_to: [admin];
            update_members => restrict_to: [admin];
            update_app => restrict_to: [admin];
            details => restrict_to: [admin];
        }
    }

    struct Member {
        admin_badge: ResourceAddress,
        member_handle: String,

        project_admins: KeyValueStore<String, Option<ComponentAddress>>,
        project_admins_total: Decimal,
        project_members: KeyValueStore<String, Option<ComponentAddress>>,
        project_members_total: Decimal,
        job_admins: KeyValueStore<String, Option<ComponentAddress>>,
        job_admins_total: Decimal,
        job_members: KeyValueStore<String, Option<ComponentAddress>>,
        job_members_total: Decimal,
        contracts: KeyValueStore<ComponentAddress, ()>,

        member_badges: KeyValueStore<ResourceAddress, ()>,
        member_components: KeyValueStore<ComponentAddress, ()>,
        apps: KeyValueStore<String, AppData>,
        resources: KeyValueStore<ResourceAddress, Vault>,
        details: KeyValueStore<String, String>,
    }

    impl Member {
        pub fn instantiate(
            dapp_address: ComponentAddress,
            member_handle: String,
        ) -> (Global<Member>, NonFungibleBucket) {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Member::blueprint_id());

            let badge_bucket = Self::nft_builder::<MemberData>(
                "Diamond Pay: Member Badge",
                "Badge containing information on the contract",
                &rule!(require(global_caller(component_address))),
                component_address,
                vec![(
                    StringNonFungibleLocalId::new(&member_handle).unwrap(),
                    MemberData {
                        member_address: component_address,
                    },
                )],
            );
            let admin_badge = badge_bucket.resource_address();

            let component = Self {
                admin_badge,
                member_handle,

                project_admins: KeyValueStore::new(),
                project_admins_total: dec!(0),
                project_members: KeyValueStore::new(),
                project_members_total: dec!(0),
                job_admins: KeyValueStore::new(),
                job_admins_total: dec!(0),
                job_members: KeyValueStore::new(),
                job_members_total: dec!(0),
                contracts: KeyValueStore::new(),

                member_badges: KeyValueStore::new(),
                member_components: KeyValueStore::new(),
                apps: KeyValueStore::new(),
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
                    "name" => "Diamond Pay: Member", locked;
                    "description" => "Creates an Member component to store account details", locked;
                    "info_url" => Url::of(INFO_URL), locked;
                    "dapp_definition" => GlobalAddress::from(dapp_address), locked;
                }
            })
            .with_address(address_reservation)
            .globalize();

            (component, badge_bucket.as_non_fungible())
        }

        pub fn add_project(&mut self, project_address: ComponentAddress) {
            assert!(
                self.contracts.get(&project_address).is_none(),
                "[Add Project]: Already Added"
            );
            let project = Global::<ProjectContract>::from(project_address);
            let role = project.role(self.admin_badge);
            let data = Some(project_address);
            if role == ContractRole::Admin {
                let new_total = self.project_admins_total + 1;
                self.project_admins_total = new_total;
                self.project_admins.insert(format!("{new_total}"), data);
            } else {
                let new_total = self.project_members_total + 1;
                self.project_members_total = new_total;
                self.project_members.insert(format!("{new_total}"), data);
            }
            self.contracts.insert(project_address, ());
        }

        pub fn add_job(&mut self, job_address: ComponentAddress) {
            assert!(
                self.contracts.get(&job_address).is_none(),
                "[Add Job]: Already Added"
            );
            let job = Global::<JobContract>::from(job_address);
            let role = job.role(self.admin_badge);
            let data = Some(job_address);
            if role == ContractRole::Admin {
                let new_total = self.job_admins_total + 1;
                self.job_admins_total = new_total;
                self.job_admins.insert(format!("{new_total}"), data);
            } else {
                let new_total = self.job_members_total + 1;
                self.job_members_total = new_total;
                self.job_members.insert(format!("{new_total}"), data);
            }
            self.contracts.insert(job_address, ());
        }

        pub fn remove_contract(&mut self, key: String, is_project: bool, is_admin: bool) {
            if is_project {
                if is_admin {
                    self.project_admins.insert(key, None);
                } else {
                    self.project_members.insert(key, None);
                }
            } else {
                if is_admin {
                    self.job_admins.insert(key, None);
                } else {
                    self.job_members.insert(key, None);
                }
            }
        }

        pub fn deposit_resource(&mut self, bucket: Bucket) {
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

        pub fn withdraw_resource(&mut self, resource_address: ResourceAddress) -> Bucket {
            self.resources
                .get_mut(&resource_address)
                .unwrap()
                .take_all()
        }

        pub fn update_members(&mut self, contacts: Vec<ResourceAddress>, is_add: bool) {
            for contact_badge in contacts {
                let global_address: GlobalAddress = ResourceManager::from(contact_badge)
                    .get_metadata("member_address")
                    .unwrap()
                    .unwrap();
                let contact_address = ComponentAddress::try_from(global_address).unwrap();
                if is_add {
                    self.member_badges.insert(contact_badge, ());
                    self.member_components.insert(contact_address, ());
                } else {
                    self.member_badges.remove(&contact_badge);
                    self.member_components.remove(&contact_address);
                }
            }
        }

        pub fn update_app(
            &mut self,
            name: String,
            data: HashMap<String, String>,
            urls: HashMap<String, Vec<Url>>,
            is_remove: bool,
        ) {
            if is_remove {
                self.apps.remove(&name);
                return;
            }

            let has_app = self.apps.get(&name).is_some();
            if has_app {
                let mut app = self.apps.get_mut(&name).unwrap();
                app.data = data;
                app.urls = urls;
            } else {
                self.apps.insert(name, AppData { data, urls });
            }
        }

        pub fn details(&mut self, details: HashMap<String, String>) {
            for (key, value) in details.iter() {
                self.details.insert(key.to_owned(), value.to_owned());
            }
        }

        // Private functions

        fn nft_builder<D: MemberRegisteredType + NonFungibleData>(
            name: &str,
            description: &str,
            access_rule: &AccessRule,
            member_address: ComponentAddress,
            entries: Vec<(StringNonFungibleLocalId, D)>,
        ) -> NonFungibleBucket {
            ResourceBuilder::new_string_non_fungible_with_registered_type::<D>(OwnerRole::None)
                .metadata(metadata! {
                    init {
                      "name" => name, locked;
                      "description" => description, locked;
                      "tags" => ["badge"], locked;
                      "icon_url" => Url::of(ICON_URL), locked;
                      "info_url" => Url::of(INFO_URL), locked;
                      "member_address" => GlobalAddress::from(member_address), locked;
                    }
                })
                .mint_roles(mint_roles! {
                    minter => access_rule.clone();
                    minter_updater => rule!(deny_all);
                })
                .mint_initial_supply(entries)
        }
    }
}
