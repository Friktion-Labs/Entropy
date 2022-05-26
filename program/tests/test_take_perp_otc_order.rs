mod program_test;

use mango::{
    matching::Side,
    state::{
        AssetType, HealthCache, HealthType, MangoAccount, MangoGroup, OtcOrderStatus, OtcOrders,
        UserActiveAssets,
    },
};
use program_test::{cookies::*, *};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

#[tokio::test]
async fn success_ask() {
    let config = MangoProgramTestConfig::default_two_mints();
    let mut test = MangoProgramTest::start_new(&config).await;

    // Initialize
    let mut mango_group_cookie = MangoGroupCookie::default(&mut test).await;
    let num_precreated_mango_users = 0;
    mango_group_cookie
        .full_setup(&mut test, num_precreated_mango_users, config.num_mints - 1)
        .await;

    let mango_group_pk = &(mango_group_cookie.address.clone());
    let mango_group = test.load_account::<MangoGroup>(*mango_group_pk).await;

    // Create `MangoAccount` for creator
    let account0_pk = test.create_mango_account(mango_group_pk, 0, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account0_pk,
        0,
        0,
        None,
    )
    .await;

    // Create `MangoAccount` for counterparty
    let account1_pk = test.create_mango_account(mango_group_pk, 1, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account1_pk,
        1,
        0,
        None,
    )
    .await;

    // Deposit funds
    let creator_amount = 4000;
    let counterparty_amount = 6000;

    let creator_deposit_amount = creator_amount * (test.quote_mint.unit as u64);
    let counterparty_deposit_amount = counterparty_amount * (test.quote_mint.unit as u64);

    mango_group_cookie.run_keeper(&mut test).await;

    test.perform_deposit_with_mango_acc_pk(
        &mango_group_cookie,
        &account0_pk,
        0,
        test.quote_index,
        creator_deposit_amount,
    )
    .await;
    test.perform_deposit_with_mango_acc_pk(
        &mango_group_cookie,
        &account1_pk,
        1,
        test.quote_index,
        counterparty_deposit_amount,
    )
    .await;

    // Create Perp OTC order
    let (creator_otc_orders_pk, _) = test.init_otc_orders(mango_group_pk, &account0_pk, 0).await;

    let price = 1;
    let size = 200;
    let expires = 9999999999999;

    let counterparty_sk = Keypair::from_bytes(&test.users[1].to_bytes()).unwrap();
    test.create_perp_otc_order(
        mango_group_pk,
        &account0_pk,
        &counterparty_sk.pubkey(),
        &mango_group_cookie.perp_markets[0].address,
        0,
        price,
        size,
        expires,
        Side::Ask,
    )
    .await
    .unwrap();

    let creator_mango_account_before = test.load_account::<MangoAccount>(account0_pk).await;
    let counterparty_mango_account_before = test.load_account::<MangoAccount>(account1_pk).await;
    assert_eq!(creator_mango_account_before.perp_accounts[0].base_position, 0);
    assert_eq!(counterparty_mango_account_before.perp_accounts[0].base_position, 0);

    let active_assets_counterparty = UserActiveAssets::new(
        &mango_group,
        &counterparty_mango_account_before,
        vec![(AssetType::Perp, 0)],
    );

    let active_assets_creator = UserActiveAssets::new(
        &mango_group,
        &creator_mango_account_before,
        vec![(AssetType::Perp, 0)],
    );

    let mut health_cache_counterparty = HealthCache::new(active_assets_counterparty);
    let mut health_cache_creator = HealthCache::new(active_assets_creator);
    assert_eq!(health_cache_counterparty.get_health(&mango_group, HealthType::Init), 0);
    assert_eq!(health_cache_creator.get_health(&mango_group, HealthType::Init), 0);

    // Execute order
    test.take_perp_otc_order(
        mango_group_pk,
        &account1_pk,
        &account0_pk,
        &mango_group_cookie.perp_markets[0].address,
        &mango_group.mango_cache,
        &Vec::new(),
        &Vec::new(),
        1,
        0,
    )
    .await
    .unwrap();

    let creator_mango_account_after = test.load_account::<MangoAccount>(account0_pk).await;
    let counterparty_mango_account_after = test.load_account::<MangoAccount>(account1_pk).await;
    assert_eq!(creator_mango_account_after.perp_accounts[0].base_position, -(size as i64));
    assert_eq!(counterparty_mango_account_after.perp_accounts[0].base_position, size as i64);

    let otc_orders = test.load_account::<OtcOrders>(creator_otc_orders_pk).await;
    assert_eq!(otc_orders.perp_orders_len, 1);
    assert_eq!(otc_orders.perp_orders[0].status, OtcOrderStatus::Filled);
}

#[tokio::test]
async fn success_bid() {
    let config = MangoProgramTestConfig::default_two_mints();
    let mut test = MangoProgramTest::start_new(&config).await;

    // Initialize
    let mut mango_group_cookie = MangoGroupCookie::default(&mut test).await;
    let num_precreated_mango_users = 0;
    mango_group_cookie
        .full_setup(&mut test, num_precreated_mango_users, config.num_mints - 1)
        .await;

    let mango_group_pk = &(mango_group_cookie.address.clone());
    let mango_group = test.load_account::<MangoGroup>(*mango_group_pk).await;

    // Create `MangoAccount` for creator
    let account0_pk = test.create_mango_account(mango_group_pk, 0, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account0_pk,
        0,
        0,
        None,
    )
    .await;

    // Create `MangoAccount` for counterparty
    let account1_pk = test.create_mango_account(mango_group_pk, 1, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account1_pk,
        1,
        0,
        None,
    )
    .await;

    // Deposit funds
    let creator_amount = 5_000;
    let counterparty_amount = 15_000;

    let creator_deposit_amount = creator_amount * (test.quote_mint.unit as u64);
    let counterparty_deposit_amount = counterparty_amount * (test.quote_mint.unit as u64);

    mango_group_cookie.run_keeper(&mut test).await;

    test.perform_deposit_with_mango_acc_pk(
        &mango_group_cookie,
        &account0_pk,
        0,
        test.quote_index,
        creator_deposit_amount,
    )
    .await;
    test.perform_deposit_with_mango_acc_pk(
        &mango_group_cookie,
        &account1_pk,
        1,
        test.quote_index,
        counterparty_deposit_amount,
    )
    .await;

    // Create Perp OTC order
    let (creator_otc_orders_pk, _) = test.init_otc_orders(mango_group_pk, &account0_pk, 0).await;

    let price = 1;
    let size = 200;
    let expires = 9999999999999;

    let counterparty_sk = Keypair::from_bytes(&test.users[1].to_bytes()).unwrap();
    test.create_perp_otc_order(
        mango_group_pk,
        &account0_pk,
        &counterparty_sk.pubkey(),
        &mango_group_cookie.perp_markets[0].address,
        0,
        price,
        size,
        expires,
        Side::Bid,
    )
    .await
    .unwrap();

    let creator_mango_account_before = test.load_account::<MangoAccount>(account0_pk).await;
    let counterparty_mango_account_before = test.load_account::<MangoAccount>(account1_pk).await;
    assert_eq!(creator_mango_account_before.perp_accounts[0].base_position, 0);
    assert_eq!(counterparty_mango_account_before.perp_accounts[0].base_position, 0);

    let active_assets_counterparty = UserActiveAssets::new(
        &mango_group,
        &counterparty_mango_account_before,
        vec![(AssetType::Perp, 0)],
    );

    let active_assets_creator = UserActiveAssets::new(
        &mango_group,
        &creator_mango_account_before,
        vec![(AssetType::Perp, 0)],
    );

    let mut health_cache_counterparty = HealthCache::new(active_assets_counterparty);
    let mut health_cache_creator = HealthCache::new(active_assets_creator);
    assert_eq!(health_cache_counterparty.get_health(&mango_group, HealthType::Init), 0);
    assert_eq!(health_cache_creator.get_health(&mango_group, HealthType::Init), 0);

    // Execute order
    test.take_perp_otc_order(
        mango_group_pk,
        &account1_pk,
        &account0_pk,
        &mango_group_cookie.perp_markets[0].address,
        &mango_group.mango_cache,
        &Vec::new(),
        &Vec::new(),
        1,
        0,
    )
    .await
    .unwrap();

    let creator_mango_account_after = test.load_account::<MangoAccount>(account0_pk).await;
    let counterparty_mango_account_after = test.load_account::<MangoAccount>(account1_pk).await;
    assert_eq!(creator_mango_account_after.perp_accounts[0].base_position, size as i64);
    assert_eq!(counterparty_mango_account_after.perp_accounts[0].base_position, -(size as i64));

    let otc_orders = test.load_account::<OtcOrders>(creator_otc_orders_pk).await;
    assert_eq!(otc_orders.perp_orders_len, 1);
    assert_eq!(otc_orders.perp_orders[0].status, OtcOrderStatus::Filled);
}

#[tokio::test]
async fn fail_invalid_account() {
    let config = MangoProgramTestConfig::default_two_mints();
    let mut test = MangoProgramTest::start_new(&config).await;

    // Initialize
    let mut mango_group_cookie = MangoGroupCookie::default(&mut test).await;
    let num_precreated_mango_users = 0;
    mango_group_cookie
        .full_setup(&mut test, num_precreated_mango_users, config.num_mints - 1)
        .await;

    let mango_group_pk = &(mango_group_cookie.address.clone());
    let mango_group = test.load_account::<MangoGroup>(*mango_group_pk).await;

    // Create `MangoAccount` for creator
    let account0_pk = test.create_mango_account(mango_group_pk, 0, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account0_pk,
        0,
        0,
        None,
    )
    .await;

    // Create `MangoAccount` for counterparty
    let account1_pk = test.create_mango_account(mango_group_pk, 1, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account1_pk,
        1,
        0,
        None,
    )
    .await;

    // Deposit funds
    let creator_amount = 4000;
    let counterparty_amount = 6000;

    let creator_deposit_amount = creator_amount * (test.quote_mint.unit as u64);
    let counterparty_deposit_amount = counterparty_amount * (test.quote_mint.unit as u64);

    mango_group_cookie.run_keeper(&mut test).await;

    test.perform_deposit_with_mango_acc_pk(
        &mango_group_cookie,
        &account0_pk,
        0,
        test.quote_index,
        creator_deposit_amount,
    )
    .await;
    test.perform_deposit_with_mango_acc_pk(
        &mango_group_cookie,
        &account1_pk,
        1,
        test.quote_index,
        counterparty_deposit_amount,
    )
    .await;

    // Create Perp OTC order
    let (_, _) = test.init_otc_orders(mango_group_pk, &account0_pk, 0).await;

    let price = 1;
    let size = 200;
    let expires = 9999999999999;

    let counterparty_sk = Keypair::from_bytes(&test.users[1].to_bytes()).unwrap();
    test.create_perp_otc_order(
        mango_group_pk,
        &account0_pk,
        &counterparty_sk.pubkey(),
        &mango_group_cookie.perp_markets[0].address,
        0,
        price,
        size,
        expires,
        Side::Ask,
    )
    .await
    .unwrap();

    // Execute order
    let error = test
        .take_perp_otc_order(
            mango_group_pk,
            &account1_pk,
            &account0_pk,
            &mango_group_cookie.perp_markets[0].address,
            &mango_group.mango_cache,
            &Vec::new(),
            &Vec::new(),
            0,
            0,
        )
        .await
        .unwrap_err();

    let error_code = get_error_code(error);
    assert_eq!(error_code, Some(25));
}

#[tokio::test]
async fn fail_insufficient_funds() {
    let config = MangoProgramTestConfig::default_two_mints();
    let mut test = MangoProgramTest::start_new(&config).await;

    // Initialize
    let mut mango_group_cookie = MangoGroupCookie::default(&mut test).await;
    let num_precreated_mango_users = 0;
    mango_group_cookie
        .full_setup(&mut test, num_precreated_mango_users, config.num_mints - 1)
        .await;

    let mango_group_pk = &(mango_group_cookie.address.clone());
    let mango_group = test.load_account::<MangoGroup>(*mango_group_pk).await;

    // Create `MangoAccount` for creator
    let account0_pk = test.create_mango_account(mango_group_pk, 0, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account0_pk,
        0,
        0,
        None,
    )
    .await;

    // Create `MangoAccount` for counterparty
    let account1_pk = test.create_mango_account(mango_group_pk, 1, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account1_pk,
        1,
        0,
        None,
    )
    .await;

    // Deposit funds
    let creator_amount = 0;
    let counterparty_amount = 0;

    let creator_deposit_amount = creator_amount * (test.quote_mint.unit as u64);
    let counterparty_deposit_amount = counterparty_amount * (test.quote_mint.unit as u64);

    mango_group_cookie.run_keeper(&mut test).await;

    test.perform_deposit_with_mango_acc_pk(
        &mango_group_cookie,
        &account0_pk,
        0,
        test.quote_index,
        creator_deposit_amount,
    )
    .await;
    test.perform_deposit_with_mango_acc_pk(
        &mango_group_cookie,
        &account1_pk,
        1,
        test.quote_index,
        counterparty_deposit_amount,
    )
    .await;

    // Create Perp OTC order
    let (_, _) = test.init_otc_orders(mango_group_pk, &account0_pk, 0).await;

    let price = 1;
    let size = 200;
    let expires = 9999999999999;

    let counterparty_sk = Keypair::from_bytes(&test.users[1].to_bytes()).unwrap();
    test.create_perp_otc_order(
        mango_group_pk,
        &account0_pk,
        &counterparty_sk.pubkey(),
        &mango_group_cookie.perp_markets[0].address,
        0,
        price,
        size,
        expires,
        Side::Ask,
    )
    .await
    .unwrap();

    // Execute order
    let error = test
        .take_perp_otc_order(
            mango_group_pk,
            &account1_pk,
            &account0_pk,
            &mango_group_cookie.perp_markets[0].address,
            &mango_group.mango_cache,
            &Vec::new(),
            &Vec::new(),
            1,
            0,
        )
        .await
        .unwrap_err();

    let error_code = get_error_code(error);
    assert_eq!(error_code, Some(7));
}

#[tokio::test]
async fn fail_otc_order_expired() {
    let config = MangoProgramTestConfig::default_two_mints();
    let mut test = MangoProgramTest::start_new(&config).await;

    // Initialize
    let mut mango_group_cookie = MangoGroupCookie::default(&mut test).await;
    let num_precreated_mango_users = 0;
    mango_group_cookie
        .full_setup(&mut test, num_precreated_mango_users, config.num_mints - 1)
        .await;

    let mango_group_pk = &(mango_group_cookie.address.clone());
    let mango_group = test.load_account::<MangoGroup>(*mango_group_pk).await;

    // Create `MangoAccount` for creator
    let account0_pk = test.create_mango_account(mango_group_pk, 0, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account0_pk,
        0,
        0,
        None,
    )
    .await;

    // Create `MangoAccount` for counterparty
    let account1_pk = test.create_mango_account(mango_group_pk, 1, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account1_pk,
        1,
        0,
        None,
    )
    .await;

    // Deposit funds
    let creator_amount = 4000;
    let counterparty_amount = 6000;

    let creator_deposit_amount = creator_amount * (test.quote_mint.unit as u64);
    let counterparty_deposit_amount = counterparty_amount * (test.quote_mint.unit as u64);

    mango_group_cookie.run_keeper(&mut test).await;

    test.perform_deposit_with_mango_acc_pk(
        &mango_group_cookie,
        &account0_pk,
        0,
        test.quote_index,
        creator_deposit_amount,
    )
    .await;
    test.perform_deposit_with_mango_acc_pk(
        &mango_group_cookie,
        &account1_pk,
        1,
        test.quote_index,
        counterparty_deposit_amount,
    )
    .await;

    // Create Perp OTC order
    let (_, _) = test.init_otc_orders(mango_group_pk, &account0_pk, 0).await;

    let clock = test.get_clock().await;

    let price = 1;
    let size = 200;
    let expires = clock.unix_timestamp + 1;

    let counterparty_sk = Keypair::from_bytes(&test.users[1].to_bytes()).unwrap();
    test.create_perp_otc_order(
        mango_group_pk,
        &account0_pk,
        &counterparty_sk.pubkey(),
        &mango_group_cookie.perp_markets[0].address,
        0,
        price,
        size,
        expires,
        Side::Ask,
    )
    .await
    .unwrap();

    test.context.warp_to_slot(1500).unwrap();

    // Execute order
    let error = test
        .take_perp_otc_order(
            mango_group_pk,
            &account1_pk,
            &account0_pk,
            &mango_group_cookie.perp_markets[0].address,
            &mango_group.mango_cache,
            &Vec::new(),
            &Vec::new(),
            1,
            0,
        )
        .await
        .unwrap_err();

    let error_code = get_error_code(error);
    assert_eq!(error_code, Some(45));
}

#[tokio::test]
async fn fail_invalid_otc_order_status() {
    let config = MangoProgramTestConfig::default_two_mints();
    let mut test = MangoProgramTest::start_new(&config).await;

    // Initialize
    let mut mango_group_cookie = MangoGroupCookie::default(&mut test).await;
    let num_precreated_mango_users = 0;
    mango_group_cookie
        .full_setup(&mut test, num_precreated_mango_users, config.num_mints - 1)
        .await;

    let mango_group_pk = &(mango_group_cookie.address.clone());
    let mango_group = test.load_account::<MangoGroup>(*mango_group_pk).await;

    // Create `MangoAccount` for creator
    let account0_pk = test.create_mango_account(mango_group_pk, 0, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account0_pk,
        0,
        0,
        None,
    )
    .await;

    // Create `MangoAccount` for counterparty
    let account1_pk = test.create_mango_account(mango_group_pk, 1, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account1_pk,
        1,
        0,
        None,
    )
    .await;

    // Deposit funds
    let creator_amount = 4000;
    let counterparty_amount = 6000;

    let creator_deposit_amount = creator_amount * (test.quote_mint.unit as u64);
    let counterparty_deposit_amount = counterparty_amount * (test.quote_mint.unit as u64);

    mango_group_cookie.run_keeper(&mut test).await;

    test.perform_deposit_with_mango_acc_pk(
        &mango_group_cookie,
        &account0_pk,
        0,
        test.quote_index,
        creator_deposit_amount,
    )
    .await;
    test.perform_deposit_with_mango_acc_pk(
        &mango_group_cookie,
        &account1_pk,
        1,
        test.quote_index,
        counterparty_deposit_amount,
    )
    .await;

    // Create Perp OTC order
    let (_, _) = test.init_otc_orders(mango_group_pk, &account0_pk, 0).await;

    let price = 1;
    let size = 200;
    let expires = 9999999999999;

    let counterparty_sk = Keypair::from_bytes(&test.users[1].to_bytes()).unwrap();
    test.create_perp_otc_order(
        mango_group_pk,
        &account0_pk,
        &counterparty_sk.pubkey(),
        &mango_group_cookie.perp_markets[0].address,
        0,
        price,
        size,
        expires,
        Side::Ask,
    )
    .await
    .unwrap();

    // Execute order
    test.take_perp_otc_order(
        mango_group_pk,
        &account1_pk,
        &account0_pk,
        &mango_group_cookie.perp_markets[0].address,
        &mango_group.mango_cache,
        &Vec::new(),
        &Vec::new(),
        1,
        0,
    )
    .await
    .unwrap();

    test.context.warp_to_slot(10).unwrap();

    let error = test
        .take_perp_otc_order(
            mango_group_pk,
            &account1_pk,
            &account0_pk,
            &mango_group_cookie.perp_markets[0].address,
            &mango_group.mango_cache,
            &Vec::new(),
            &Vec::new(),
            1,
            0,
        )
        .await
        .unwrap_err();

    let error_code = get_error_code(error);
    assert_eq!(error_code, Some(43));
}
