//! The client contains the logic to launch the Tor hidden service and encapsulates
//! services like the database, onion service,...

/// Encapsulates hidden service, database connection,...
pub struct Client {
    onion_service: std::sync::Arc<tor_hsservice::RunningOnionService>,
}
