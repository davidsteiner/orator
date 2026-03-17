use crate::api::generated::{HealthApi, HealthCheckResponse};
use std::convert::Infallible;
use tennis_club_core::TennisClub;

impl HealthApi for TennisClub {
    type Error = Infallible;

    async fn health_check(&self, _ctx: ()) -> Result<HealthCheckResponse, Self::Error> {
        Ok(HealthCheckResponse::Ok("ok".to_string()))
    }
}
