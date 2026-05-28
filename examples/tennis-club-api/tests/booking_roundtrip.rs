//! Verifies that serializing a discriminated-enum variant produces JSON
//! with exactly one copy of the discriminator field. See orator issue
//! about `#[serde(tag = ...)]` double-emit when the body struct also
//! declares the discriminator property.

use tennis_club_api::api::generated::types::{Booking, GuestBooking, MemberBooking};

#[test]
fn booking_serialize_emits_single_booking_type_key() {
    let booking = Booking::Member(MemberBooking {
        booking_type: "member".to_string(),
        court_id: orator_axum::uuid::Uuid::nil(),
        member_id: "11111111-1111-1111-1111-111111111111".parse().unwrap(),
        date: orator_axum::chrono::NaiveDate::from_ymd_opt(2026, 5, 28).unwrap(),
    });

    let json = orator_axum::serde_json::to_string(&booking).expect("Booking should serialize");

    let count = json.matches("\"booking_type\":").count();
    assert_eq!(
        count, 1,
        "expected exactly one booking_type key in {json}, got {count}",
    );
}

#[test]
fn booking_round_trips_through_json() {
    let original = Booking::Member(MemberBooking {
        booking_type: "member".to_string(),
        court_id: orator_axum::uuid::Uuid::nil(),
        member_id: "11111111-1111-1111-1111-111111111111".parse().unwrap(),
        date: orator_axum::chrono::NaiveDate::from_ymd_opt(2026, 5, 28).unwrap(),
    });

    let json = orator_axum::serde_json::to_string(&original).expect("Booking should serialize");
    let parsed: Booking =
        orator_axum::serde_json::from_str(&json).expect("Booking should deserialize");

    assert_eq!(original, parsed, "round-trip should preserve value");
}

#[test]
fn guest_booking_round_trips_through_json() {
    let original = Booking::Guest(GuestBooking {
        booking_type: "guest".to_string(),
        court_id: orator_axum::uuid::Uuid::nil(),
        guest_name: "Pat".to_string(),
        date: orator_axum::chrono::NaiveDate::from_ymd_opt(2026, 5, 28).unwrap(),
    });

    let json = orator_axum::serde_json::to_string(&original).expect("Booking should serialize");
    let parsed: Booking =
        orator_axum::serde_json::from_str(&json).expect("Booking should deserialize");

    assert_eq!(original, parsed, "round-trip should preserve value");
}

#[test]
fn booking_deserialize_rejects_unknown_discriminator() {
    let wire = r#"{"booking_type":"corporate","court_id":"00000000-0000-0000-0000-000000000000","date":"2026-05-28"}"#;
    let result: Result<Booking, _> = orator_axum::serde_json::from_str(wire);
    assert!(
        result.is_err(),
        "deserialize should reject an unknown discriminator value",
    );
}
