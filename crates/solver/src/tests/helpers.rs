use crate::solve_impl;
use crate::types::{
    Corr, Image, Pixel, SolveRequest, SolveResponse, WorldLla,
};

pub(crate) fn sample_corr(id: &str, u: f64, v: f64, lat: f64, lon: f64, alt: f64) -> Corr {
    Corr {
        id: id.to_string(),
        enabled: Some(true),
        pixel: Pixel {
            u,
            v,
            sigma_px: Some(1.0),
        },
        world: WorldLla {
            lat,
            lon,
            alt: Some(alt),
        },
    }
}

pub(crate) fn base_request(correspondences: Vec<Corr>) -> SolveRequest {
    SolveRequest {
        image: Image {
            width: 4000.0,
            height: 3000.0,
        },
        correspondences,
        priors: None,
    }
}

pub(crate) fn solve_to_response(req: SolveRequest) -> SolveResponse {
    solve_impl(&req).expect("solve must succeed")
}

pub(crate) fn solve_impl_error(req: SolveRequest) -> String {
    match solve_impl(&req) {
        Ok(_) => panic!("expected solve_impl to fail"),
        Err(msg) => msg,
    }
}

pub(crate) fn haversine_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6_371_000.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}
