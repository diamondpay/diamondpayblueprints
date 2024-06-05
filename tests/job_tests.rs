use diamondpay::job_contract::job_contract_test::JobContractState;
use scrypto_test::prelude::*;
mod common;

fn create_job(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    admin: common::MemberData,
    resource_address: ResourceAddress,
    start: i64,
    cliff: Option<i64>,
    end: i64,
    interval: i64,
) -> ComponentAddress {
    let public_key = admin.public_key;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            admin.account_address,
            admin.resource_address,
            vec![admin.lid.clone()],
        )
        .pop_from_auth_zone("proof")
        .call_function_with_name_lookup(package_address, "JobContract", "instantiate", |lookup| {
            (
                "app_handle",
                "contract_handle",
                "Contract Name",
                admin.resource_address,
                lookup.proof("proof"),
                resource_address,
                start,
                cliff,
                end,
                interval,
            )
        })
        .call_method(
            admin.account_address,
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
    components[0]
}

fn job_test(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    job_address: ComponentAddress,
    method_name: &str,
    args: impl ResolvableArguments,
) {
    let public_key = member.public_key;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            member.account_address,
            member.resource_address,
            vec![member.lid.clone()],
        )
        .call_method(job_address, method_name, args)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

fn job_cancellation(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    component_address: ComponentAddress,
) {
    let public_key = member.public_key;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            member.account_address,
            member.resource_address,
            vec![member.lid.clone()],
        )
        .call_method(component_address, "cancellation", manifest_args!())
        .call_method(
            member.account_address,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

fn job_join(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    job_address: ComponentAddress,
) {
    let public_key = member.public_key;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            member.account_address,
            member.resource_address,
            vec![member.lid.clone()],
        )
        .pop_from_auth_zone("proof")
        .call_method_with_name_lookup(job_address, "join", |lookup| {
            (member.resource_address, lookup.proof("proof"))
        })
        .call_method(
            member.account_address,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

fn job_leave(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    job_address: ComponentAddress,
) {
    let public_key = member.public_key;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            member.account_address,
            member.resource_address,
            vec![member.lid.clone()],
        )
        .pop_from_auth_zone("proof")
        .call_method_with_name_lookup(job_address, "leave", |lookup| {
            (member.resource_address, lookup.proof("proof"))
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

fn job_withdraw(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    job_address: ComponentAddress,
) {
    let public_key = member.public_key;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            member.account_address,
            member.resource_address,
            vec![member.lid.clone()],
        )
        .pop_from_auth_zone("proof")
        .call_method_with_name_lookup(job_address, "withdraw", |lookup| {
            (member.resource_address, lookup.proof("proof"))
        })
        .call_method(
            member.account_address,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

fn job_deposit(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    resource_address: ResourceAddress,
    amount: Decimal,
    job_address: ComponentAddress,
) {
    let public_key = member.public_key;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            member.account_address,
            member.resource_address,
            vec![member.lid.clone()],
        )
        .call_method(
            member.account_address,
            "withdraw",
            manifest_args!(resource_address, amount),
        )
        .take_from_worktop(resource_address, amount, "bucket1")
        .call_method_with_name_lookup(job_address, "deposit", |lookup| (lookup.bucket("bucket1"),))
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

// Comment out check_join() in order to test vesting
#[test]
fn test_members() {
    let (mut test_runner, app) = common::setup_test();
    let resource_address = test_runner.create_freely_mintable_and_burnable_fungible_resource(
        OwnerRole::None,
        Some(dec!(10500)),
        3u8,
        app.admin.account_address,
    );
    let job_address = create_job(
        &mut test_runner,
        app.package_address,
        app.admin.clone(),
        resource_address,
        1662700716i64,
        Some(1694236716i64),
        1725859156i64,
        // 1792176036i64,
        14i64,
    );
    println!("Job Address: {:?}", job_address);

    job_test(
        &mut test_runner,
        app.admin.clone(),
        job_address,
        "invite",
        manifest_args!(app.member.resource_address, "handle_2"),
    );
    job_test(
        &mut test_runner,
        app.admin.clone(),
        job_address,
        "remove",
        manifest_args!(app.member.resource_address),
    );
    job_test(
        &mut test_runner,
        app.admin.clone(),
        job_address,
        "invite",
        manifest_args!(app.member.resource_address, "handle_2"),
    );
    job_join(&mut test_runner, app.member.clone(), job_address);

    job_deposit(
        &mut test_runner,
        app.admin.clone(),
        resource_address,
        dec!(10000),
        job_address,
    );
    job_withdraw(&mut test_runner, app.member.clone(), job_address);
    job_leave(&mut test_runner, app.member.clone(), job_address);
}

// Comment out check_join() in order to test vesting
#[test]
fn test() {
    let (mut test_runner, app) = common::setup_test();
    let resource_address = test_runner.create_freely_mintable_and_burnable_fungible_resource(
        OwnerRole::None,
        Some(dec!(10500)),
        3u8,
        app.admin.account_address,
    );
    let job_address = create_job(
        &mut test_runner,
        app.package_address,
        app.admin.clone(),
        resource_address,
        1662700716i64,
        Some(1694236716i64),
        1725859156i64,
        // 1792176036i64,
        14i64,
    );
    println!("Job Address: {:?}", job_address);

    job_deposit(
        &mut test_runner,
        app.admin.clone(),
        resource_address,
        dec!(10000),
        job_address,
    );

    job_test(
        &mut test_runner,
        app.admin.clone(),
        job_address,
        "invite",
        manifest_args!(app.member.resource_address, app.member.handle.clone()),
    );
    job_join(&mut test_runner, app.member.clone(), job_address);

    job_withdraw(&mut test_runner, app.member.clone(), job_address);
    job_leave(&mut test_runner, app.member.clone(), job_address);
    job_cancellation(&mut test_runner, app.admin.clone(), job_address);

    let j_state: JobContractState = test_runner.component_state(job_address);
    println!("State: {:?}", j_state.vesting_schedule.withdrawn);
    assert!(j_state.vesting_schedule.withdrawn == dec!(4979.477));
}
