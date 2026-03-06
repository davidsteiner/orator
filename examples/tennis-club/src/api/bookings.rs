use crate::TennisClub;
use crate::api::generated::{BookingsApi, ListBookingsResponse};
use std::convert::Infallible;

impl BookingsApi for TennisClub {
    type Error = Infallible;

    async fn list_bookings(&self, _ctx: ()) -> Result<ListBookingsResponse, Self::Error> {
        let bookings = self.bookings.lock().unwrap();
        Ok(ListBookingsResponse::Ok(bookings.clone()))
    }
}
