use policy_maintainer::check_alloy_provider_invariant::{
    AlloyProviderEvaluation, evaluate_cargo_tree_output,
};

#[test]
fn alloy_provider_invariant_treats_missing_package_as_success() {
    let evaluation = evaluate_cargo_tree_output(
        false,
        "",
        "error: package ID specification `alloy-provider` did not match any packages",
    );

    assert_eq!(evaluation, AlloyProviderEvaluation::Holds);
}

#[test]
fn alloy_provider_invariant_accepts_allowlisted_dependents() {
    let evaluation = evaluate_cargo_tree_output(
        true,
        "alloy-provider v2.0.4\n├── cow-sdk-alloy-provider v0.1.0\n└── cow-sdk-alloy v0.1.0\n",
        "",
    );

    assert_eq!(evaluation, AlloyProviderEvaluation::Holds);
}

#[test]
fn alloy_provider_invariant_reports_unexpected_dependents_as_violation() {
    let evaluation = evaluate_cargo_tree_output(
        true,
        "alloy-provider v2.0.4\n└── cow-sdk-trading v0.1.0\n",
        "",
    );

    assert!(matches!(
        evaluation,
        AlloyProviderEvaluation::Violated(detail) if detail.contains("cow-sdk-trading")
    ));
}
