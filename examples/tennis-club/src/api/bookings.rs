use crate::TennisClub;
use crate::api::generated::{BookingsApi, ListBookingsParams, ListBookingsResponse};
use std::convert::Infallible;

impl BookingsApi for TennisClub {
    type Error = Infallible;

    async fn list_bookings(
        &self,
        _ctx: (),
        _params: ListBookingsParams,
    ) -> Result<ListBookingsResponse, Self::Error> {
        let bookings = self.bookings.lock().unwrap();
        Ok(ListBookingsResponse::Ok(bookings.clone()))
    }
}
