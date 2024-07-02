use scrypto_test::prelude::*;
mod common;

fn member_test(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
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
        .call_method(member.member_component, method_name, args)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

fn member_deposit(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    resource_address: ResourceAddress,
    amount: Decimal,
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
        .call_method_with_name_lookup(member.member_component, "deposit", |lookup| {
            (lookup.bucket("bucket1"),)
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

fn member_withdraw(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    resource_address: ResourceAddress,
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
            member.member_component,
            "withdraw",
            manifest_args!(resource_address),
        )
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

#[test]
fn test() {
    let (mut test_runner, app) = common::setup_test();
    member_test(
        &mut test_runner,
        app.admin.clone(),
        "details",
        manifest_args!(
            HashMap::from([("description", "New Description")]),
            "https://google.com",
            true,
        ),
    );
    member_test(
        &mut test_runner,
        app.admin.clone(),
        "details",
        manifest_args!(
            HashMap::from([("description", "New Description 2")]),
            "https://reddit.com",
            false
        ),
    );
    member_deposit(
        &mut test_runner,
        app.admin.clone(),
        app.resource_address,
        dec!(3000),
    );
    member_withdraw(&mut test_runner, app.admin.clone(), app.resource_address);
    member_test(
        &mut test_runner,
        app.admin.clone(),
        "update_members",
        manifest_args!(vec!(app.member.resource_address), false),
    );
    member_test(
        &mut test_runner,
        app.admin.clone(),
        "update_members",
        manifest_args!(vec!(app.member.resource_address), true),
    );
    member_test(
        &mut test_runner,
        app.admin.clone(),
        "update_team",
        manifest_args!(
            "App Name",
            HashMap::from([("description", "Test description goes here")]),
            None::<ResourceAddress>
        ),
    );
    member_test(
        &mut test_runner,
        app.admin.clone(),
        "update_team",
        manifest_args!(
            "App Name",
            HashMap::from([("description", "Updated here")]),
            Some(app.member.resource_address)
        ),
    );
    member_test(
        &mut test_runner,
        app.admin.clone(),
        "remove_team",
        manifest_args!("App Name",),
    );
}
