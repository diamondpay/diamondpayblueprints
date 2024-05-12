use scrypto::this_package;
use scrypto_test::prelude::*;
use scrypto_unit::*;

#[derive(Clone)]
pub struct MemberData {
    pub lid: NonFungibleLocalId,
    pub account_address: ComponentAddress,
    pub public_key: Secp256k1PublicKey,
    pub resource_address: ResourceAddress,
    pub handle: String,
}

#[derive(Clone)]
pub struct TestSetup {
    pub package_address: PackageAddress,
    pub admin: MemberData,
    pub member: MemberData,
}

fn create_env() -> (
    TestRunner<NoExtension, InMemorySubstateDatabase>,
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
    let mut test_runner = TestRunnerBuilder::new()
        .without_trace()
        .with_custom_genesis(config)
        .build();
    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());
    (test_runner, package_address)
}

fn create_member(
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    handle: &str,
) -> MemberData {
    // Create an account
    let (public_key, _, account_address) = test_runner.new_allocated_account();

    let id_str = StringNonFungibleLocalId::new(handle).unwrap();
    let lid = NonFungibleLocalId::String(id_str);
    let resource_address = test_runner.create_freely_mintable_and_burnable_non_fungible_resource(
        OwnerRole::None,
        NonFungibleIdType::String,
        Some(vec![(lid.clone(), ())]),
        account_address,
    );

    let member_data = MemberData {
        lid,
        account_address,
        public_key,
        resource_address,
        handle: handle.to_string(),
    };

    member_data
}

pub fn setup_test() -> (TestRunner<NoExtension, InMemorySubstateDatabase>, TestSetup) {
    let (mut test_runner, package_address) = create_env();
    let admin = create_member(&mut test_runner, "handle_1");
    let member = create_member(&mut test_runner, "handle_2");

    let app_setup = TestSetup {
        package_address,
        admin,
        member,
    };
    (test_runner, app_setup)
}
