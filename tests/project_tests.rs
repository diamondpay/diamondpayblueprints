use diamondpay::project_contract::project_contract_test::ProjectContractState;
use scrypto_test::prelude::*;
mod common;

fn create_project(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    admin: common::MemberData,
    resource_address: ResourceAddress,
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
        .call_function_with_name_lookup(
            package_address,
            "ProjectContract",
            "instantiate",
            |lookup| {
                (
                    admin.account_address,
                    Some(admin.member_component),
                    admin.resource_address,
                    lookup.proof("proof"),
                    "team_handle",
                    "contract_handle",
                    "Contract Name",
                    resource_address,
                    1662700716i64,
                    1725859156i64,
                    HashMap::from([
                        ("obj_names", ""),
                        ("description", "Test description goes here"),
                        ("social_urls", "https://google.com"),
                        ("link_urls", "https://google.com"),
                        ("image_urls", "https://google.com"),
                        ("video_ids", "id1"),
                    ]),
                )
            },
        )
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

fn project_test(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    project_address: ComponentAddress,
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
        .call_method(project_address, method_name, args)
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

fn project_leave(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    project_address: ComponentAddress,
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
        .call_method_with_name_lookup(project_address, "leave", |lookup| {
            (member.resource_address, lookup.proof("proof"))
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

fn project_join(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    project_address: ComponentAddress,
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
        .call_method_with_name_lookup(project_address, "join", |lookup| {
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

fn project_deposit(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    resource_address: ResourceAddress,
    amount: Decimal,
    project_address: ComponentAddress,
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
        .call_method_with_name_lookup(project_address, "deposit", |lookup| {
            (lookup.bucket("bucket1"),)
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

fn project_cancellation(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    project_address: ComponentAddress,
) {
    let public_key = member.public_key;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
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
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

fn project_withdraw(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    project_address: ComponentAddress,
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
        .call_method_with_name_lookup(project_address, "withdraw", |lookup| {
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

fn project_list(
    test_runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    member: common::MemberData,
    project_address: ComponentAddress,
    marketplace_address: ComponentAddress,
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
            manifest_args!(XRD, dec!(100)), // fee
        )
        .take_from_worktop(XRD, dec!(100), "bucket1")
        .call_method(
            project_address,
            "list",
            manifest_args!(marketplace_address, "Test"),
        )
        .pop_from_auth_zone("proof")
        .call_method_with_name_lookup(marketplace_address, "add_project", |lookup| {
            (
                "Test",
                project_address,
                lookup.proof("proof"),
                lookup.bucket("bucket1"),
            )
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

//
//
// Tests

#[test]
fn test_members() {
    let (mut test_runner, app) = common::setup_test();
    let project_address = create_project(
        &mut test_runner,
        app.package_address,
        app.admin.clone(),
        app.resource_address,
    );
    println!("Project Address: {:?}", project_address);

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
    // Toggle this to test List vs Cancellation/Withdraw
    let is_list = true;

    let (mut test_runner, app) = common::setup_test();
    // Marketplace add market
    project_test(
        &mut test_runner,
        app.admin.clone(),
        app.marketplace_address,
        "add_markets",
        manifest_args!(vec!["Main"], dec!(2000), dec!(100), app.resource_address),
    );
    project_test(
        &mut test_runner,
        app.admin.clone(),
        app.marketplace_address,
        "update_market",
        manifest_args!(
            "Main",
            true,
            dec!(3000),
            HashMap::from([("description", "Test")])
        ),
    );

    let project_address = create_project(
        &mut test_runner,
        app.package_address,
        app.admin.clone(),
        app.resource_address,
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
    project_test(
        &mut test_runner,
        app.member.clone(),
        app.member.member_component,
        "add_project",
        manifest_args!(project_address),
    );

    project_deposit(
        &mut test_runner,
        app.admin.clone(),
        app.resource_address,
        dec!(3000),
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
        (
            dec!(5),
            HashMap::from([(app.member.resource_address, dec!(200))]),
        ),
        (
            dec!(6),
            HashMap::from([(app.member.resource_address, dec!(1000))]),
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
        "details",
        manifest_args!(
            1662700716i64,
            1725859156i64,
            HashMap::from([("obj_names", "Objective 1, Objective 2, Objective 3")]),
            true
        ),
    );
    if is_list {
        project_list(
            &mut test_runner,
            app.admin.clone(),
            project_address,
            app.marketplace_address,
        );
        project_test(
            &mut test_runner,
            app.admin.clone(),
            app.marketplace_address,
            "withdraw",
            manifest_args!(XRD),
        );
        project_test(
            &mut test_runner,
            app.admin.clone(),
            app.marketplace_address,
            "remove_contract",
            manifest_args!(project_address, "Test", true),
        );
    }

    project_test(
        &mut test_runner,
        app.admin.clone(),
        project_address,
        "reward",
        manifest_args!(dec!(1)),
    );
    if !is_list {
        project_withdraw(&mut test_runner, app.admin.clone(), project_address);
        project_withdraw(&mut test_runner, app.member.clone(), project_address);
    }
    project_test(
        &mut test_runner,
        app.admin.clone(),
        project_address,
        "reward",
        manifest_args!(dec!(2)),
    );
    project_test(
        &mut test_runner,
        app.admin.clone(),
        project_address,
        "reward",
        manifest_args!(dec!(3)),
    );

    if !is_list {
        project_cancellation(&mut test_runner, app.admin.clone(), project_address);
        project_withdraw(&mut test_runner, app.admin.clone(), project_address);
        project_withdraw(&mut test_runner, app.member.clone(), project_address);
    }

    let p_state: ProjectContractState = test_runner.component_state(project_address);
    assert!(p_state.completed.contains_key(&dec!(1)));
    assert!(p_state.completed.contains_key(&dec!(2)));
    assert!(p_state.completed.contains_key(&dec!(3)));
}
