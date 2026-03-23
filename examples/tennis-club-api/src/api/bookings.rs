use crate::api::generated::operations::{BookingsApi, ListBookingsResponse};
use std::convert::Infallible;
use tennis_club_core::{TennisClub, domain};

impl BookingsApi for TennisClub {
    type Error = Infallible;
    type RequestContext = ();

    async fn list_bookings(&self, _ctx: ()) -> Result<ListBookingsResponse, Self::Error> {
        let bookings = self.list_bookings();
        let bookings = bookings.into_iter().map(Into::into).collect();
        Ok(ListBookingsResponse::Ok(bookings))
    }
}

impl From<domain::Booking> for crate::api::generated::types::Booking {
    fn from(b: domain::Booking) -> Self {
        match b {
            domain::Booking::MemberBooking(data) => {
                Self::MemberBooking(crate::api::generated::types::MemberBooking {
                    booking_type: "member".to_string(),
                    court_id: data.court_id,
                    member_id: data.member_id,
                    date: data.date,
                })
            }
            domain::Booking::GuestBooking(data) => {
                Self::GuestBooking(crate::api::generated::types::GuestBooking {
                    booking_type: "guest".to_string(),
                    court_id: data.court_id,
                    guest_name: data.guest_name,
                    date: data.date,
                })
            }
        }
    }
}
