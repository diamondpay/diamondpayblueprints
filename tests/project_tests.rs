use diamondpay::project_contract::test_bindings::*;
use scrypto_test::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
mod common;

fn create_project(
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    admin: common::MemberData,
    resource_address: ResourceAddress,
) -> ComponentAddress {
    let public_key = admin.public_key;
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            admin.account_address,
            admin.resource_address,
            vec![admin.lid.clone()],
        )
        .pop_from_auth_zone("proof")
        .call_function_with_name_lookup(
            package_address,
            "ProjectContract",
            "instantiate",
            |lookup| {
                (
                    "app_handle",
                    "contract_handle",
                    "Contract Name",
                    admin.resource_address,
                    lookup.proof("proof"),
                    resource_address,
                )
            },
        )
        .call_method(
            admin.account_address,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let outcome = receipt.expect_commit_success();
    let components = outcome.new_component_addresses();
    components[0]
}

fn project_test(
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    project_address: ComponentAddress,
    method_name: &str,
    args: impl ResolvableArguments,
) {
    let public_key = member.public_key;
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            member.account_address,
            member.resource_address,
            vec![member.lid.clone()],
        )
        .call_method(project_address, method_name, args)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

fn project_leave(
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    project_address: ComponentAddress,
) {
    let public_key = member.public_key;
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            member.account_address,
            member.resource_address,
            vec![member.lid.clone()],
        )
        .pop_from_auth_zone("proof")
        .call_method_with_name_lookup(project_address, "leave", |lookup| {
            (member.resource_address, lookup.proof("proof"))
        })
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

fn project_join(
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    project_address: ComponentAddress,
) {
    let public_key = member.public_key;
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            member.account_address,
            member.resource_address,
            vec![member.lid.clone()],
        )
        .pop_from_auth_zone("proof")
        .call_method_with_name_lookup(project_address, "join", |lookup| {
            (member.resource_address, lookup.proof("proof"))
        })
        .call_method(
            member.account_address,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

fn project_deposit(
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    resource_address: ResourceAddress,
    amount: Decimal,
    project_address: ComponentAddress,
) {
    let public_key = member.public_key;
    let manifest = ManifestBuilder::new()
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
        .call_method_with_name_lookup(project_address, "deposit", |lookup| {
            (lookup.bucket("bucket1"),)
        })
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

fn project_cancellation(
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    project_address: ComponentAddress,
) {
    let public_key = member.public_key;
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            member.account_address,
            member.resource_address,
            vec![member.lid.clone()],
        )
        .call_method(project_address, "cancellation", manifest_args!())
        .call_method(
            member.account_address,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

fn project_withdraw(
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    project_address: ComponentAddress,
) {
    let public_key = member.public_key;
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            member.account_address,
            member.resource_address,
            vec![member.lid.clone()],
        )
        .pop_from_auth_zone("proof")
        .call_method_with_name_lookup(project_address, "withdraw", |lookup| {
            (member.resource_address, lookup.proof("proof"))
        })
        .call_method(
            member.account_address,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

//
//
// Tests

#[test]
fn test_members() {
    let (mut test_runner, app) = common::setup_test();
    let resource_address = test_runner.create_freely_mintable_and_burnable_fungible_resource(
        OwnerRole::None,
        Some(dec!(1000)),
        3u8,
        app.admin.account_address,
    );
    let project_address = create_project(
        &mut test_runner,
        app.package_address,
        app.admin.clone(),
        resource_address,
    );
    project_test(
        &mut test_runner,
        app.admin.clone(),
        project_address,
        "invite",
        manifest_args!(app.member.resource_address, "handle_2"),
    );
    project_test(
        &mut test_runner,
        app.admin.clone(),
        project_address,
        "remove",
        manifest_args!(app.member.resource_address),
    );
    project_test(
        &mut test_runner,
        app.admin.clone(),
        project_address,
        "invite",
        manifest_args!(app.member.resource_address, "handle_2"),
    );
    project_join(&mut test_runner, app.member.clone(), project_address);
    project_leave(&mut test_runner, app.member.clone(), project_address);
}

#[test]
fn test() {
    let (mut test_runner, app) = common::setup_test();
    let resource_address = test_runner.create_freely_mintable_and_burnable_fungible_resource(
        OwnerRole::None,
        Some(dec!(10000)),
        3u8,
        app.admin.account_address,
    );
    let project_address = create_project(
        &mut test_runner,
        app.package_address,
        app.admin.clone(),
        resource_address,
    );
    println!("Project Address: {:?}", project_address);

    project_test(
        &mut test_runner,
        app.admin.clone(),
        project_address,
        "invite",
        manifest_args!(app.admin.resource_address, "handle_1"),
    );
    project_join(&mut test_runner, app.admin.clone(), project_address);

    project_test(
        &mut test_runner,
        app.admin.clone(),
        project_address,
        "invite",
        manifest_args!(app.member.resource_address, "handle_2"),
    );
    project_join(&mut test_runner, app.member.clone(), project_address);
    project_deposit(
        &mut test_runner,
        app.admin.clone(),
        resource_address,
        dec!(1800),
        project_address,
    );
    let objs = HashMap::from([
        (
            dec!(1),
            HashMap::from([
                (app.admin.resource_address, dec!(100)),
                (app.member.resource_address, dec!(200)),
            ]),
        ),
        (
            dec!(2),
            HashMap::from([
                (app.admin.resource_address, dec!(200)),
                (app.member.resource_address, dec!(400)),
            ]),
        ),
        (
            dec!(3),
            HashMap::from([
                (app.admin.resource_address, dec!(100)),
                (app.member.resource_address, dec!(200)),
            ]),
        ),
        (
            dec!(4),
            HashMap::from([
                (app.admin.resource_address, dec!(200)),
                (app.member.resource_address, dec!(400)),
            ]),
        ),
    ]);
    project_test(
        &mut test_runner,
        app.admin.clone(),
        project_address,
        "update",
        manifest_args!(objs),
    );
    project_test(
        &mut test_runner,
        app.admin.clone(),
        project_address,
        "complete",
        manifest_args!(dec!(1)),
    );
    project_withdraw(&mut test_runner, app.admin.clone(), project_address);
    project_withdraw(&mut test_runner, app.member.clone(), project_address);
    project_test(
        &mut test_runner,
        app.admin.clone(),
        project_address,
        "complete",
        manifest_args!(dec!(2)),
    );
    project_test(
        &mut test_runner,
        app.admin.clone(),
        project_address,
        "complete",
        manifest_args!(dec!(3)),
    );
    project_cancellation(&mut test_runner, app.admin.clone(), project_address);
    project_withdraw(&mut test_runner, app.admin.clone(), project_address);
    project_withdraw(&mut test_runner, app.member.clone(), project_address);

    let p_state: ProjectContractState = test_runner.component_state(project_address);
    assert!(p_state.completed.contains_key(&dec!(1)));
    assert!(p_state.completed.contains_key(&dec!(2)));
    assert!(p_state.completed.contains_key(&dec!(3)));
}
