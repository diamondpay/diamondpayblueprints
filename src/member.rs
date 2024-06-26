use crate::contract_types::*;
use crate::job_contract::job_contract::JobContract;
use crate::marketplace::marketplace::Marketplace;
use crate::project_contract::project_contract::ProjectContract;
use scrypto::prelude::*;

#[blueprint]
#[types(MemberData, String, TeamData)]
mod member {
    enable_method_auth! {
        roles {
            admin => updatable_by: [];
        },
        methods {
            add_project => PUBLIC;
            add_job => PUBLIC;
            remove_contract => restrict_to: [admin];
            deposit => restrict_to: [admin];
            withdraw => restrict_to: [admin];
            update_members => restrict_to: [admin];
            update_team => restrict_to: [admin];
            remove_team => restrict_to: [admin];
            details => restrict_to: [admin];
            get_badge => PUBLIC;
        }
    }

    struct Member {
        admin_badge: ResourceAddress,
        badge_manager: ResourceManager,
        member_handle: String,
        marketplace: Global<Marketplace>,

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
        apps: KeyValueStore<String, TeamData>,
        resources: KeyValueStore<ResourceAddress, Vault>,
        details: KeyValueStore<String, String>,
    }

    impl Member {
        pub fn instantiate(
            dapp_address: ComponentAddress,
            member_handle: String,
            icon_url: String,
            markets: Vec<String>,
        ) -> (Global<Member>, NonFungibleBucket) {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Member::blueprint_id());

            let badge_bucket = Self::nft_builder::<MemberData>(
                "Diamond Pay: Member Badge",
                "Badge used for contracts and authentication",
                &rule!(require(global_caller(component_address))),
                component_address,
                &icon_url,
                vec![(
                    StringNonFungibleLocalId::new(&member_handle).unwrap(),
                    MemberData {
                        member_address: component_address,
                    },
                )],
            );
            let admin_badge = badge_bucket.resource_address();

            let marketplace = Marketplace::instantiate(
                admin_badge,
                member_handle.clone(),
                dapp_address,
                markets,
                dec!(0),
                dec!(0),
                XRD,
            );

            let component = Self {
                admin_badge,
                badge_manager: badge_bucket.resource_manager(),
                member_handle,
                marketplace,

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
                apps: KeyValueStore::<String, TeamData>::new_with_registered_type(),
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

        pub fn update_members(&mut self, contacts: Vec<ResourceAddress>, is_remove: bool) {
            for contact_badge in contacts {
                let contact = Self::badge_to_member(contact_badge);
                if is_remove {
                    self.member_badges.remove(&contact_badge);
                    self.member_components.remove(&contact.address());
                } else {
                    self.member_badges.insert(contact_badge, ());
                    self.member_components.insert(contact.address(), ());
                }
            }
        }

        pub fn update_team(
            &mut self,
            name: String,
            details: HashMap<String, String>,
            team_badge: Option<ResourceAddress>,
        ) {
            if team_badge.is_some() {
                // verify that badge is a Member badge
                Self::badge_to_member(team_badge.unwrap());
            }

            let has_team = self.apps.get(&name).is_some();
            if has_team {
                // update while preserving insert order
                let mut app = self.apps.get_mut(&name).unwrap();
                app.details = details;
                app.team_badge = team_badge;
            } else {
                self.apps.insert(
                    name,
                    TeamData {
                        details,
                        team_badge,
                    },
                );
            }
        }

        pub fn remove_team(&mut self, name: String) {
            self.apps.remove(&name);
        }

        pub fn details(&mut self, details: HashMap<String, String>, icon_url: String) {
            for (key, value) in details.iter() {
                self.details.insert(key.to_owned(), value.to_owned());
            }
            self.badge_manager
                .set_metadata("icon_url", Url::of(icon_url));
        }

        pub fn get_badge(&self) -> ResourceAddress {
            self.admin_badge
        }

        // Private functions

        fn nft_builder<D: MemberRegisteredType + NonFungibleData>(
            name: &str,
            description: &str,
            access_rule: &AccessRule,
            member_address: ComponentAddress,
            icon_url: &str,
            entries: Vec<(StringNonFungibleLocalId, D)>,
        ) -> NonFungibleBucket {
            ResourceBuilder::new_string_non_fungible_with_registered_type::<D>(OwnerRole::None)
                .metadata(metadata! {
                    roles {
                        metadata_setter => access_rule.clone();
                        metadata_setter_updater => rule!(deny_all);
                        metadata_locker => rule!(deny_all);
                        metadata_locker_updater => rule!(deny_all);
                    },
                    init {
                      "name" => name, locked;
                      "description" => description, locked;
                      "tags" => ["badge"], locked;
                      "icon_url" => Url::of(icon_url), updatable;
                      "info_url" => Url::of(INFO_URL), locked;
                      MEMBER_ADDRESS => GlobalAddress::from(member_address), locked;
                    }
                })
                .mint_roles(mint_roles! {
                    minter => access_rule.clone();
                    minter_updater => rule!(deny_all);
                })
                .mint_initial_supply(entries)
        }

        // use member_address from metadata to avoid doing extra gateway api call to get nft data
        fn badge_to_member(contact_badge: ResourceAddress) -> Global<Member> {
            let global_address: GlobalAddress = ResourceManager::from(contact_badge)
                .get_metadata(MEMBER_ADDRESS)
                .unwrap()
                .unwrap();
            let contact_address = ComponentAddress::try_from(global_address).unwrap();
            let contact = Global::<Member>::from(contact_address);
            assert!(
                contact_badge == contact.get_badge(),
                "[Badge to Member]: Invalid"
            );
            contact
        }
    }
}
