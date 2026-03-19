use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Member {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewMember {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateMember {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Court {
    pub id: Uuid,
    pub name: String,
    pub surface: Surface,
    pub indoor: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Surface {
    Clay,
    Grass,
    Hard,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "booking_type")]
pub enum Booking {
    #[serde(rename = "member")]
    MemberBooking(MemberBookingData),
    #[serde(rename = "guest")]
    GuestBooking(GuestBookingData),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemberBookingData {
    pub court_id: Uuid,
    pub member_id: Uuid,
    pub date: NaiveDate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuestBookingData {
    pub court_id: Uuid,
    pub guest_name: String,
    pub date: NaiveDate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Error {
    pub code: i32,
    pub message: String,
}
