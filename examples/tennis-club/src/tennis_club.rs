use crate::api::generated::{Booking, Court, GuestBooking, Member, MemberBooking, Surface};
use std::sync::Mutex;

pub struct TennisClub {
    pub members: Mutex<Vec<Member>>,
    pub courts: Mutex<Vec<Court>>,
    pub bookings: Mutex<Vec<Booking>>,
    pub next_id: Mutex<i64>,
}

impl TennisClub {
    pub fn new() -> Self {
        let courts = vec![
            Court {
                id: 1,
                name: "Court 1".to_string(),
                surface: Surface::Clay,
                indoor: Some(false),
            },
            Court {
                id: 2,
                name: "Court 2".to_string(),
                surface: Surface::Hard,
                indoor: Some(true),
            },
            Court {
                id: 3,
                name: "Court 3".to_string(),
                surface: Surface::Grass,
                indoor: Some(false),
            },
        ];

        let bookings = vec![
            Booking::MemberBooking(MemberBooking {
                booking_type: "member".to_string(),
                court_id: 1,
                member_id: 1,
                date: "2026-03-10".to_string(),
            }),
            Booking::GuestBooking(GuestBooking {
                booking_type: "guest".to_string(),
                court_id: 2,
                guest_name: "Radagast the Brown".to_string(),
                date: "2026-03-11".to_string(),
            }),
        ];

        let members = vec![
            Member {
                id: 1,
                first_name: "Lobelia".to_string(),
                last_name: "Sackville-Baggins".to_string(),
            },
            Member {
                id: 2,
                first_name: "Fredegar".to_string(),
                last_name: "Bolger".to_string(),
            },
            Member {
                id: 3,
                first_name: "Folco".to_string(),
                last_name: "Boffin".to_string(),
            },
            Member {
                id: 4,
                first_name: "Estella".to_string(),
                last_name: "Brandybuck".to_string(),
            },
            Member {
                id: 5,
                first_name: "Elanor".to_string(),
                last_name: "Gamgee".to_string(),
            },
        ];

        Self {
            members: Mutex::new(members),
            courts: Mutex::new(courts),
            bookings: Mutex::new(bookings),
            next_id: Mutex::new(6),
        }
    }
}
