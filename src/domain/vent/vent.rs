#[derive(Clone, Debug)]
pub enum VentStatus {
    Disconnected,
    Connected,
    Open,
    Closed,
}