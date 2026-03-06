use crate::TennisClub;
use crate::api::generated::{CourtsApi, ListCourtsResponse};
use std::convert::Infallible;

impl CourtsApi for TennisClub {
    type Error = Infallible;

    async fn list_courts(&self, _ctx: ()) -> Result<ListCourtsResponse, Self::Error> {
        let courts = self.courts.lock().unwrap();
        Ok(ListCourtsResponse::Ok(courts.clone()))
    }
}
