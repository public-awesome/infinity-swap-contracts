use crate::setup::templates::{setup_infinity_test, standard_minter_template, InfinityTestSetup};

use infinity_factory::msg::SudoMsg as InfinityFactorySudoMsg;
use infinity_factory::msg::{QueryMsg as InfinityFactoryQueryMsg, UnrestrictedMigrationsResponse};
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn try_unrestricted_migrations() {
    let vt = standard_minter_template(1000u32);
    let InfinityTestSetup {
        vending_template: MinterTemplateResponse {
            mut router,
            ..
        },
        infinity_factory,
        ..
    } = setup_infinity_test(vt).unwrap();

    let add_unrestricted_migration_msg = InfinityFactorySudoMsg::AddUnrestrictedMigration {
        starting_code_id: 0,
        target_code_id: 1,
    };
    let response = router.wasm_sudo(infinity_factory.clone(), &add_unrestricted_migration_msg);
    assert!(response.is_ok());

    let unrestricted_migrations = router
        .wrap()
        .query_wasm_smart::<UnrestrictedMigrationsResponse>(
            infinity_factory.clone(),
            &InfinityFactoryQueryMsg::UnrestrictedMigrations {
                query_options: None,
            },
        )
        .unwrap();

    assert_eq!(unrestricted_migrations.len(), 1);
    assert_eq!(unrestricted_migrations[0], (0, 1));

    let add_unrestricted_migration_msg = InfinityFactorySudoMsg::AddUnrestrictedMigration {
        starting_code_id: 1,
        target_code_id: 3,
    };
    let response = router.wasm_sudo(infinity_factory.clone(), &add_unrestricted_migration_msg);
    assert!(response.is_ok());

    let unrestricted_migrations = router
        .wrap()
        .query_wasm_smart::<UnrestrictedMigrationsResponse>(
            infinity_factory.clone(),
            &InfinityFactoryQueryMsg::UnrestrictedMigrations {
                query_options: None,
            },
        )
        .unwrap();

    assert_eq!(unrestricted_migrations.len(), 2);
    assert_eq!(unrestricted_migrations, [(0, 1), (1, 3)]);

    let remove_unrestricted_migration_msg = InfinityFactorySudoMsg::RemoveUnrestrictedMigration {
        starting_code_id: 1,
    };
    let response = router.wasm_sudo(infinity_factory.clone(), &remove_unrestricted_migration_msg);
    assert!(response.is_ok());

    let unrestricted_migrations = router
        .wrap()
        .query_wasm_smart::<UnrestrictedMigrationsResponse>(
            infinity_factory,
            &InfinityFactoryQueryMsg::UnrestrictedMigrations {
                query_options: None,
            },
        )
        .unwrap();

    assert_eq!(unrestricted_migrations.len(), 1);
    assert_eq!(unrestricted_migrations[0], (0, 1));
}
