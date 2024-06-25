use scrypto_test::prelude::*;

#[derive(Clone)]
pub struct MemberData {
    pub lid: NonFungibleLocalId,
    pub account_address: ComponentAddress,
    pub public_key: Secp256k1PublicKey,
    pub resource_address: ResourceAddress,
    pub handle: String,
    pub member_component: ComponentAddress,
}

#[derive(Clone)]
pub struct TestSetup {
    pub package_address: PackageAddress,
    pub admin: MemberData,
    pub member: MemberData,
    pub resource_address: ResourceAddress,
    pub marketplace_address: ComponentAddress,
}

fn create_env() -> (
    LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    PackageAddress,
) {
    // Test job contract by setting initial time
    let mut config = CustomGenesis::default(
        Epoch::of(1u64),
        CustomGenesis::default_consensus_manager_config(),
    );
    // Wed Sep 20 2023
    config.initial_time_ms = 1695236716000i64;
    // Setup the environment
    let mut test_runner = LedgerSimulatorBuilder::new()
        .without_kernel_trace()
        .with_custom_genesis(config)
        .build();
    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());
    (test_runner, package_address)
}

fn create_member(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    handle: &str,
    package_address: PackageAddress,
) -> MemberData {
    // Create an account
    let (public_key, _, account_address) = test_runner.new_allocated_account();

    let id_str = StringNonFungibleLocalId::new(handle).unwrap();
    let lid = NonFungibleLocalId::String(id_str);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Member",
            "instantiate",
            manifest_args!(account_address, handle),
        )
        .call_method(
            account_address,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let outcome = receipt.expect_commit_success();
    let components = outcome.new_component_addresses();
    let resources = outcome.new_resource_addresses();

    let member_data = MemberData {
        lid,
        account_address,
        public_key,
        resource_address: resources[0],
        handle: handle.to_owned(),
        member_component: components[0],
    };

    member_data
}

fn create_marketplace(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    resource_address: ResourceAddress,
    admin: MemberData,
) -> ComponentAddress {
    let public_key = admin.public_key;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Marketplace",
            "instantiate",
            manifest_args!(
                admin.resource_address,
                "App",
                admin.account_address,
                vec!("Test", "Test2"),
                resource_address
            ),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let outcome = receipt.expect_commit_success();
    let components = outcome.new_component_addresses();
    components[0]
}

pub fn setup_test() -> (
    LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    TestSetup,
) {
    let (mut test_runner, package_address) = create_env();
    let admin = create_member(&mut test_runner, "handle_1", package_address);
    let member = create_member(&mut test_runner, "handle_2", package_address);
    let resource_address = test_runner.create_freely_mintable_and_burnable_fungible_resource(
        OwnerRole::None,
        Some(dec!(11000)),
        3u8,
        admin.account_address,
    );
    let marketplace_address = create_marketplace(
        &mut test_runner,
        package_address,
        resource_address,
        admin.clone(),
    );

    let app_setup = TestSetup {
        package_address,
        admin,
        member,
        resource_address,
        marketplace_address,
    };
    (test_runner, app_setup)
}
