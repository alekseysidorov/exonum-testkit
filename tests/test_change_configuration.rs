extern crate exonum;
extern crate exonum_testkit;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use exonum::helpers::{Height, ValidatorId};
use exonum_testkit::{TestKitBuilder, TestNetworkConfigurationBuilder};
use exonum::blockchain::Schema;
use exonum::storage::StorageValue;

#[test]
fn test_add_to_validators() {
    let mut testkit = TestKitBuilder::auditor().with_validators(1).create();

    let cfg_change_height = Height(5);
    let proposal = {
        let mut cfg_builder = TestNetworkConfigurationBuilder::new(&testkit);
        cfg_builder.validators().push(testkit.network().us().clone());
        *cfg_builder.actual_from() = cfg_change_height;
        cfg_builder.create()
    };
    let stored = proposal.stored_configuration().clone();
    testkit.commit_configuration_change(proposal);

    testkit.create_blocks_until(cfg_change_height);

    assert_eq!(testkit.network().us().validator_id(), Some(ValidatorId(1)));
    assert_eq!(&testkit.network().validators()[1], testkit.network().us());
    assert_eq!(
        Schema::new(&testkit.snapshot()).actual_configuration(),
        stored
    );
    assert_eq!(
        Schema::new(&testkit.snapshot())
            .previous_configuration()
            .unwrap()
            .hash(),
        stored.previous_cfg_hash
    );
}

#[test]
fn test_exclude_from_validators() {
    let mut testkit = TestKitBuilder::validator().with_validators(2).create();

    let cfg_change_height = Height(5);
    let proposal = {
        let mut cfg = testkit.configuration_change_proposal();
        let validator = cfg.validators()[1].clone();
        cfg.set_actual_from(cfg_change_height);
        cfg.set_validators(vec![validator]);
        cfg
    };
    let stored = proposal.stored_configuration().clone();
    testkit.commit_configuration_change(proposal);

    testkit.create_blocks_until(cfg_change_height);

    assert_eq!(testkit.network().us().validator_id(), None);
    assert_eq!(testkit.network().validators().len(), 1);
    assert_eq!(
        Schema::new(&testkit.snapshot()).actual_configuration(),
        stored
    );
    assert_eq!(
        Schema::new(&testkit.snapshot())
            .previous_configuration()
            .unwrap()
            .hash(),
        stored.previous_cfg_hash
    );
}

#[test]
fn test_change_service_config() {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct ServiceConfig {
        name: String,
        value: u64,
    };

    let service_cfg = ServiceConfig {
        name: String::from("Config"),
        value: 64,
    };

    let mut testkit = TestKitBuilder::validator().create();
    let cfg_change_height = Height(5);
    let proposal = {
        let mut cfg = testkit.configuration_change_proposal();
        cfg.set_service_config("my_service", service_cfg.clone());
        cfg.set_actual_from(cfg_change_height);
        cfg
    };
    testkit.commit_configuration_change(proposal);

    testkit.create_blocks_until(cfg_change_height);

    assert_eq!(
        serde_json::to_value(service_cfg).unwrap(),
        Schema::new(&testkit.snapshot())
            .actual_configuration()
            .services["my_service"]
    );
}
