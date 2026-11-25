use proto123_testkit::{check_scenario, scenarios};

#[test]
fn every_shared_scenario_passes_on_the_core() {
    for scenario in scenarios::all() {
        if let Err(message) = check_scenario(&scenario) {
            panic!("{message}");
        }
    }
}
