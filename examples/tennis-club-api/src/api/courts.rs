use crate::api::generated::{
    CourtsApi, ListCourtsResponse, UploadCourtPhotoBody, UploadCourtPhotoPath,
    UploadCourtPhotoResponse,
};
use std::convert::Infallible;
use tennis_club_core::{TennisClub, domain};

impl CourtsApi for TennisClub {
    type Error = Infallible;

    async fn list_courts(&self, _ctx: ()) -> Result<ListCourtsResponse, Self::Error> {
        let courts = self.list_courts();
        let courts = courts.into_iter().map(Into::into).collect();
        Ok(ListCourtsResponse::Ok(courts))
    }

    async fn upload_court_photo(
        &self,
        _ctx: (),
        _path: UploadCourtPhotoPath,
        _body: UploadCourtPhotoBody,
    ) -> Result<UploadCourtPhotoResponse, Self::Error> {
        Ok(UploadCourtPhotoResponse::NoContent)
    }
}

impl From<domain::Court> for crate::api::generated::Court {
    fn from(c: domain::Court) -> Self {
        Self {
            id: c.id,
            name: c.name,
            surface: c.surface.into(),
            indoor: c.indoor,
        }
    }
}

impl From<domain::Surface> for crate::api::generated::Surface {
    fn from(s: domain::Surface) -> Self {
        match s {
            domain::Surface::Clay => Self::Clay,
            domain::Surface::Grass => Self::Grass,
            domain::Surface::Hard => Self::Hard,
        }
    }
}
