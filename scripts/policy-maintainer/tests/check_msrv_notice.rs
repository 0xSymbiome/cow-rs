use policy_maintainer::check_msrv_notice::{MsrvNotice, validate_notice};

#[test]
fn msrv_notice_passes_when_not_enforced_locally() {
    let notice = MsrvNotice {
        rust_version: "1.94.0".to_owned(),
        age_days: Some(3),
        enforce: false,
    };

    assert!(validate_notice(&notice).is_empty());
}

#[test]
fn msrv_notice_fails_when_release_readiness_window_is_short() {
    let notice = MsrvNotice {
        rust_version: "1.94.0".to_owned(),
        age_days: Some(3),
        enforce: true,
    };

    assert!(validate_notice(&notice)[0].contains("requires at least 30 days"));
}
