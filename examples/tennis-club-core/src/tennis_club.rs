use chrono::{NaiveDate, Utc};

use crate::domain::*;
use std::sync::Mutex;

pub struct TennisClub {
    members: Mutex<Vec<Member>>,
    courts: Mutex<Vec<Court>>,
    bookings: Mutex<Vec<Booking>>,
    next_id: Mutex<i64>,
}

impl TennisClub {
    pub fn new() -> Self {
        let now = Utc::now();
        let members = vec![
            Member {
                id: 1,
                first_name: "Lobelia".to_string(),
                last_name: "Sackville-Baggins".to_string(),
                joined_at: now,
            },
            Member {
                id: 2,
                first_name: "Fredegar".to_string(),
                last_name: "Bolger".to_string(),
                joined_at: now,
            },
            Member {
                id: 3,
                first_name: "Folco".to_string(),
                last_name: "Boffin".to_string(),
                joined_at: now,
            },
            Member {
                id: 4,
                first_name: "Estella".to_string(),
                last_name: "Brandybuck".to_string(),
                joined_at: now,
            },
            Member {
                id: 5,
                first_name: "Elanor".to_string(),
                last_name: "Gamgee".to_string(),
                joined_at: now,
            },
        ];

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
            Booking::MemberBooking(MemberBookingData {
                court_id: 1,
                member_id: 1,
                date: NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
            }),
            Booking::GuestBooking(GuestBookingData {
                court_id: 2,
                guest_name: "Radagast the Brown".to_string(),
                date: NaiveDate::from_ymd_opt(2026, 3, 11).unwrap(),
            }),
        ];

        Self {
            members: Mutex::new(members),
            courts: Mutex::new(courts),
            bookings: Mutex::new(bookings),
            next_id: Mutex::new(6),
        }
    }

    pub fn list_members(&self, limit: Option<i32>, offset: Option<i32>) -> Vec<Member> {
        let members = self.members.lock().unwrap();
        let offset = offset.unwrap_or(0) as usize;
        members
            .iter()
            .skip(offset)
            .take(limit.unwrap_or(i32::MAX) as usize)
            .cloned()
            .collect()
    }

    pub fn create_member(&self, new: NewMember) -> Member {
        let mut members = self.members.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();

        let member = Member {
            id: *next_id,
            first_name: new.first_name,
            last_name: new.last_name,
            joined_at: Utc::now(),
        };

        *next_id += 1;
        members.push(member.clone());
        member
    }

    pub fn get_member(&self, id: i64) -> Option<Member> {
        let members = self.members.lock().unwrap();
        members.iter().find(|m| m.id == id).cloned()
    }

    pub fn update_member(&self, id: i64, update: UpdateMember) -> Option<Member> {
        let mut members = self.members.lock().unwrap();
        let member = members.iter_mut().find(|m| m.id == id)?;

        if let Some(first_name) = update.first_name {
            member.first_name = first_name;
        }
        if let Some(last_name) = update.last_name {
            member.last_name = last_name;
        }

        Some(member.clone())
    }

    pub fn delete_member(&self, id: i64) -> bool {
        let mut members = self.members.lock().unwrap();
        let len_before = members.len();
        members.retain(|m| m.id != id);
        members.len() < len_before
    }

    pub fn list_courts(&self) -> Vec<Court> {
        self.courts.lock().unwrap().clone()
    }

    pub fn list_bookings(&self) -> Vec<Booking> {
        self.bookings.lock().unwrap().clone()
    }
}

impl Default for TennisClub {
    fn default() -> Self {
        Self::new()
    }
}
