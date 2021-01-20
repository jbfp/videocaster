use rocket::{
    response::{Responder, Result as ResponseResult},
    Request,
};
use rocket_contrib::json::Json;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct AppResult<T: Serialize> {
    success: bool,
    obj: Option<T>,
    error: Option<String>,
}

impl<T: Serialize, E: ToString> From<Result<T, E>> for AppResult<T> {
    fn from(r: Result<T, E>) -> Self {
        match r {
            Ok(t) => AppResult {
                success: true,
                obj: Some(t),
                error: None,
            },
            Err(e) => AppResult {
                success: false,
                obj: None,
                error: Some(e.to_string()),
            },
        }
    }
}

impl<'r, T: Serialize> Responder<'r, 'static> for AppResult<T> {
    fn respond_to(self, request: &'r Request<'_>) -> ResponseResult<'static> {
        if let Some(err) = self.error.as_deref() {
            error!("Error: {}", err);
        }

        Json(self).respond_to(request)
    }
}
