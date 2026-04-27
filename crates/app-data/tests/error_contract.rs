use cow_sdk_app_data::AppDataError;
use cow_sdk_core::Cancelled;

#[test]
fn cancelled_marker_lifts_to_app_data_error_cancelled() {
    fn assert_from_cancelled<T: From<Cancelled>>() {}

    assert_from_cancelled::<AppDataError>();
    assert!(matches!(
        AppDataError::from(Cancelled),
        AppDataError::Cancelled
    ));
}
