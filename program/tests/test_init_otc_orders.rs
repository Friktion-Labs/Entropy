mod program_test;

use mango::state::{OtcOrderStatus, OtcOrders};
use program_test::{cookies::*, *};
use solana_program_test::*;

#[tokio::test]
async fn success_init_otc_orders() {
    let config = MangoProgramTestConfig::default_two_mints();
    let mut test = MangoProgramTest::start_new(&config).await;

    let mut mango_group_cookie = MangoGroupCookie::default(&mut test).await;
    let num_precreated_mango_users = 0;
    mango_group_cookie
        .full_setup(&mut test, num_precreated_mango_users, config.num_mints - 1)
        .await;

    let mango_group_pk = &mango_group_cookie.address;

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

    let (otc_orders_pk, bump) = test.init_otc_orders(mango_group_pk, &account0_pk, 0).await;
    let otc_orders = test.load_account::<OtcOrders>(otc_orders_pk).await;

    assert_eq!(otc_orders.meta_data.data_type, 12);
    assert!(otc_orders.meta_data.is_initialized);
    assert_eq!(otc_orders.creator_account, account0_pk);
    assert_eq!(otc_orders.perp_orders[0].status, OtcOrderStatus::Uninitialized);
    assert_eq!(otc_orders.spot_orders[0].status, OtcOrderStatus::Uninitialized);
    assert_eq!(otc_orders.perp_orders_len, 0);
    assert_eq!(otc_orders.spot_orders_len, 0);
    assert_eq!(otc_orders.bump, bump);
}
