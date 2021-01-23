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

impl<T: Serialize> AppResult<T> {
    pub(crate) fn success(t: T) -> Self {
        Self {
            success: true,
            obj: Some(t),
            error: None,
        }
    }

    pub(crate) fn error<E: ToString>(e: E) -> Self {
        Self {
            success: false,
            obj: None,
            error: Some(e.to_string()),
        }
    }
}

impl<T: Serialize, E: ToString> From<Result<T, E>> for AppResult<T> {
    fn from(r: Result<T, E>) -> Self {
        match r {
            Ok(t) => Self::success(t),
            Err(e) => Self::error(e),
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
