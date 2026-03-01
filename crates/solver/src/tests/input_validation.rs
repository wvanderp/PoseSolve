use super::helpers::{base_request, sample_corr, solve_impl_error};

#[test]
fn solve_rejects_empty_correspondence_list() {
    let req = base_request(vec![]);
    let msg = solve_impl_error(req);
    assert!(msg.contains("Need at least one enabled correspondence"));
}

#[test]
fn solve_rejects_when_all_points_disabled() {
    let mut corr = sample_corr("p1", 100.0, 200.0, 51.9, 4.46, 12.0);
    corr.enabled = Some(false);
    let req = base_request(vec![corr]);
    let msg = solve_impl_error(req);
    assert!(msg.contains("Need at least one enabled correspondence"));
}
