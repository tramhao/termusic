use std::sync::Arc;

use axum::extract::State;
use axum::http::{self, Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

use super::AppState;

#[derive(Clone)]
pub struct Auth {}

pub async fn auth<B>(
    State(state): State<Arc<AppState>>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, (StatusCode, &'static str)> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header: &str = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err((
            StatusCode::UNAUTHORIZED,
            "AUTHORIZATION token can't be empty!",
        ));
    };

    if auth_header == state.web_service_token {
        // insert the current user into a request extension so the handler can
        // extract it
        req.extensions_mut().insert(Auth {});
        Ok(next.run(req).await)
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            "AUTHORIZATION token is not valid!",
        ))
    }
}
