use crate::contract_types::*;
use crate::job_contract::job_contract::JobContract;
use crate::list::list::List;
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
            project_request => PUBLIC;
            job_request => PUBLIC;
            remove_contract => restrict_to: [admin];
            remove_request => restrict_to: [admin];
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

        project_admins: Owned<List>,
        project_members: Owned<List>,
        project_requests: Owned<List>,
        job_admins: Owned<List>,
        job_members: Owned<List>,
        job_requests: Owned<List>,

        member_badges: KeyValueStore<ResourceAddress, ()>,
        member_components: KeyValueStore<ComponentAddress, ()>,
        apps: KeyValueStore<String, TeamData>,
        resources: KeyValueStore<ResourceAddress, Vault>,
        is_any_invite: bool,
        details: KeyValueStore<String, String>,
    }

    impl Member {
        pub fn instantiate(
            dapp_address: ComponentAddress,
            member_handle: String,
            icon_url: String,
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

            let component = Self {
                admin_badge,
                badge_manager: badge_bucket.resource_manager(),
                member_handle,

                project_admins: List::new(),
                project_members: List::new(),
                project_requests: List::new(),
                job_admins: List::new(),
                job_members: List::new(),
                job_requests: List::new(),

                member_badges: KeyValueStore::new(),
                member_components: KeyValueStore::new(),
                apps: KeyValueStore::<String, TeamData>::new_with_registered_type(),
                resources: KeyValueStore::new(),
                is_any_invite: false,
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
            let project = Global::<ProjectContract>::from(project_address);
            let role = project.role(self.admin_badge);
            match role {
                ContractRole::Admin => self.project_admins.add(project_address),
                ContractRole::Member => self.project_members.add(project_address),
                ContractRole::Nonmember => Runtime::panic(String::from("[Contract]: Not a member")),
            }
        }

        pub fn add_job(&mut self, job_address: ComponentAddress) {
            let job = Global::<JobContract>::from(job_address);
            let role = job.role(self.admin_badge);
            match role {
                ContractRole::Admin => self.job_admins.add(job_address),
                ContractRole::Member => self.job_members.add(job_address),
                ContractRole::Nonmember => Runtime::panic(String::from("[Contract]: Not a member")),
            }
        }

        pub fn project_request(
            &mut self,
            proof: NonFungibleProof,
            project_address: ComponentAddress,
        ) {
            let proof_badge = proof.resource_address();
            let has_badge = self.member_badges.get(&proof_badge).is_some();
            assert!(
                has_badge || self.is_any_invite,
                "[Request]: Not in Contacts"
            );

            let project = Global::<ProjectContract>::from(project_address);
            let proof_role = project.role(proof_badge);
            assert!(
                proof_role == ContractRole::Admin,
                "[Request]: Not contract admin"
            );
            let member_role = project.role(self.admin_badge);
            assert!(
                member_role == ContractRole::Nonmember,
                "[Request]: Already a member"
            );
            self.project_requests.add(project_address);
        }

        pub fn job_request(&mut self, proof: NonFungibleProof, job_address: ComponentAddress) {
            let proof_badge = proof.resource_address();
            let has_badge = self.member_badges.get(&proof_badge).is_some();
            assert!(
                has_badge || self.is_any_invite,
                "[Request]: Not in Contacts"
            );

            let job = Global::<JobContract>::from(job_address);
            let proof_role = job.role(proof_badge);
            assert!(
                proof_role == ContractRole::Admin,
                "[Request]: Not contract admin"
            );
            let member_role = job.role(self.admin_badge);
            assert!(
                member_role == ContractRole::Nonmember,
                "[Request]: Already a member"
            );
            self.job_requests.add(job_address);
        }

        pub fn remove_contract(
            &mut self,
            address: ComponentAddress,
            is_project: bool,
            is_admin: bool,
        ) {
            if is_project {
                if is_admin {
                    self.project_admins.remove(address);
                } else {
                    self.project_members.remove(address);
                }
            } else {
                if is_admin {
                    self.job_admins.remove(address);
                } else {
                    self.job_members.remove(address);
                }
            }
        }

        pub fn remove_request(&mut self, address: ComponentAddress, is_project: bool) {
            if is_project {
                self.project_requests.remove(address);
            } else {
                self.job_requests.remove(address);
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

        pub fn details(
            &mut self,
            details: HashMap<String, String>,
            icon_url: String,
            is_any_invite: bool,
        ) {
            for (key, value) in details.iter() {
                self.details.insert(key.to_owned(), value.to_owned());
            }
            self.badge_manager
                .set_metadata("icon_url", Url::of(icon_url));
            self.is_any_invite = is_any_invite;
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
