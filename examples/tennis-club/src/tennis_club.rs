use crate::api::generated::{Court, Member, Surface};
use std::sync::Mutex;

pub struct TennisClub {
    pub members: Mutex<Vec<Member>>,
    pub courts: Mutex<Vec<Court>>,
    pub next_id: Mutex<i64>,
}

impl TennisClub {
    pub fn new() -> Self {
        let courts = vec![
            Court {
                id: 1,
                name: "Centre Court".to_string(),
                surface: Surface::Grass,
                indoor: Some(false),
            },
            Court {
                id: 2,
                name: "Court Philippe-Chatrier".to_string(),
                surface: Surface::Clay,
                indoor: Some(false),
            },
            Court {
                id: 3,
                name: "Indoor Hard Court".to_string(),
                surface: Surface::Hard,
                indoor: Some(true),
            },
        ];

        Self {
            members: Mutex::new(Vec::new()),
            courts: Mutex::new(courts),
            next_id: Mutex::new(1),
        }
    }
}
