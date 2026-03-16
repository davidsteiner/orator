use crate::api::generated::{CourtsApi, ListCourtsResponse};
use std::convert::Infallible;
use tennis_club::TennisClub;

impl CourtsApi for TennisClub {
    type Error = Infallible;

    async fn list_courts(&self, _ctx: ()) -> Result<ListCourtsResponse, Self::Error> {
        let courts = self.list_courts();
        let courts = courts.into_iter().map(Into::into).collect();
        Ok(ListCourtsResponse::Ok(courts))
    }
}

impl From<tennis_club::domain::Court> for crate::api::generated::Court {
    fn from(c: tennis_club::domain::Court) -> Self {
        Self {
            id: c.id,
            name: c.name,
            surface: c.surface.into(),
            indoor: c.indoor,
        }
    }
}

impl From<tennis_club::domain::Surface> for crate::api::generated::Surface {
    fn from(s: tennis_club::domain::Surface) -> Self {
        match s {
            tennis_club::domain::Surface::Clay => Self::Clay,
            tennis_club::domain::Surface::Grass => Self::Grass,
            tennis_club::domain::Surface::Hard => Self::Hard,
        }
    }
}
