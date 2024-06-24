use crate::contract_types::*;
use scrypto::prelude::*;

#[blueprint]
#[types(AppData, MemberData)]
mod member {
    enable_method_auth! {
        roles {
            admin => updatable_by: [];
        },
        methods {
            deposit_resource => restrict_to: [admin];
            withdraw_resource => restrict_to: [admin];
            deposit_contract => restrict_to: [admin];
            withdraw_contract => restrict_to: [admin];
            update_members => restrict_to: [admin];
            update_app => restrict_to: [admin];
            details => restrict_to: [admin];
        }
    }

    struct Member {
        admin_badge: ResourceAddress,
        member_handle: String,

        project_admins: KeyValueStore<ResourceAddress, NonFungibleVault>,
        project_members: KeyValueStore<ResourceAddress, NonFungibleVault>,
        job_admins: KeyValueStore<ResourceAddress, NonFungibleVault>,
        job_members: KeyValueStore<ResourceAddress, NonFungibleVault>,

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
            details: HashMap<String, String>,
        ) -> (Global<Member>, NonFungibleBucket) {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Member::blueprint_id());

            let badge_manager = Self::nft_builder::<MemberData>(
                &format!("Diamond Pay: Member Badge"),
                &format!("Badge containing information on the contract"),
                &rule!(require(global_caller(component_address))),
                component_address,
            );
            let admin_badge = badge_manager.address();
            let badge_bucket = badge_manager.mint_non_fungible(
                &Self::nft_id(member_handle.clone()),
                MemberData {
                    member_address: component_address,
                },
            );

            let new_details = KeyValueStore::<String, String>::new();
            for (key, value) in details.iter() {
                new_details.insert(key.to_owned(), value.to_owned());
            }

            let component = Self {
                admin_badge,
                member_handle,

                project_admins: KeyValueStore::new(),
                project_members: KeyValueStore::new(),
                job_admins: KeyValueStore::new(),
                job_members: KeyValueStore::new(),

                member_badges: KeyValueStore::new(),
                member_components: KeyValueStore::new(),
                apps: KeyValueStore::new(),
                resources: KeyValueStore::new(),
                details: new_details,
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

        // All contract nfts are unique, do not need to check if Vault already exists
        pub fn deposit_contract(&mut self, bucket: NonFungibleBucket) {
            let nfts = bucket.non_fungibles::<BadgeData>();
            let vault = NonFungibleVault::with_bucket(bucket);

            for nft in nfts {
                let resource_address = nft.resource_address();
                let data = nft.data();
                if data.contract_kind == ContractKind::Project {
                    if data.contract_role == ContractRole::Admin {
                        self.project_admins.insert(resource_address, vault);
                        return;
                    } else {
                        self.project_members.insert(resource_address, vault);
                        return;
                    }
                } else {
                    if data.contract_role == ContractRole::Admin {
                        self.job_admins.insert(resource_address, vault);
                        return;
                    } else {
                        self.job_members.insert(resource_address, vault);
                        return;
                    }
                }
            }
        }

        pub fn withdraw_contract(
            &mut self,
            resource_address: ResourceAddress,
            is_project: bool,
            is_admin: bool,
        ) -> NonFungibleBucket {
            if is_project {
                if is_admin {
                    self.project_admins
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all()
                } else {
                    self.project_members
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all()
                }
            } else {
                if is_admin {
                    self.job_admins
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all()
                } else {
                    self.job_members
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all()
                }
            }
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
        ) -> ResourceManager {
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
                .create_with_no_initial_supply()
        }

        fn nft_id(id: String) -> NonFungibleLocalId {
            let nft_str = StringNonFungibleLocalId::new(id).unwrap();
            NonFungibleLocalId::String(nft_str)
        }
    }
}
