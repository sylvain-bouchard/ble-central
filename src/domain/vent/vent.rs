#[derive(Clone, Copy, serde::Serialize)]
pub enum VentState {
    Open,
    Closed,
}