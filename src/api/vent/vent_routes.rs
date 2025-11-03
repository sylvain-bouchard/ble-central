use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};

use crate::{domain::vent::vent::VentState, state::ApplicationState};

pub fn build_router(state: ApplicationState) -> Router {
    Router::new()
        .route("/vent/open", post(open_vent))
        .route("/vent/close", post(close_vent))
        .route("/vent/status", get(vent_status))
        .with_state(state)
}

async fn open_vent(State(state): State<ApplicationState>) -> &'static str {
    state.vent.open();
    "Vent opened"
}

async fn close_vent(State(state): State<ApplicationState>) -> &'static str {
    state.vent.close();
    "Vent closed"
}

async fn vent_status(State(state): State<ApplicationState>) -> Json<VentState> {
    Json(state.vent.status())
}
