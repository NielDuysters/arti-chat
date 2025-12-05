//! The client contains the logic to launch the Tor hidden service and encapsulates
//! services like the database, onion service,...

use arti_client::config::onion_service::OnionServiceConfigBuilder;
use crate::error;
use futures::Stream;
use tokio::sync::Mutex as TokioMutex;

type ArtiTorClient = arti_client::TorClient<tor_rtcompat::PreferredRuntime>;
type OnionServiceRequestStream = Box<dyn Stream<Item = tor_hsservice::RendRequest> + Send>;
type DatabaseConnection = std::sync::Arc<TokioMutex<rusqlite::Connection>>; 

/// Encapsulates hidden service, database connection,...
pub struct Client {
    /// Arti Tor Client.
    pub tor_client: ArtiTorClient,

    /// Running hidden onion service.
    onion_service: std::sync::Arc<tor_hsservice::RunningOnionService>,

    /// Request stream of onion service to handle incoming requests.
    /// We have to store it seperately because we can't derive it after `launch_onion_service`.
    request_stream: TokioMutex<std::pin::Pin<OnionServiceRequestStream>>,

    /// Database connection.
    db_conn: DatabaseConnection,
}

impl Client {
    /// Launch chat client.
    /// Bootstrap TorClient + launch onion service + ...
    pub async fn launch(db_conn: DatabaseConnection) -> Result<Self, error::ClientError> {
        // Create Tor Client.
        let tor_client = Self::bootstrap_tor_client().await?;

        // Launch onion service.
        let (onion_service, request_stream) = Self::launch_onion_service(&tor_client).await?;
        let request_stream = TokioMutex::new(Box::into_pin(request_stream));

        tracing::info!("ArtiChat client launched.");

        Ok(Self {
            tor_client,
            onion_service,
            request_stream,
            db_conn,
        })
    }

    /// Get onion service identity unredacted.
    /// Warning: This displays the full hidden service onion url.
    pub fn get_identity_unredacted(&self) -> Result<String, error::ClientError> {
        self.onion_service
            .onion_address()
            .ok_or(error::ClientError::EmptyHsid)
            .map(|address| safelog::DispUnredacted(address).to_string())
    }

    async fn bootstrap_tor_client() -> Result<ArtiTorClient, error::ClientError> {
        let config = arti_client::TorClientConfig::default();
        let client = arti_client::TorClient::create_bootstrapped(config).await?;

        tracing::info!("Tor Client bootstrapped.");

        Ok(client)
    }
    
    async fn launch_onion_service(client: &ArtiTorClient)
        -> Result<(
            std::sync::Arc<tor_hsservice::RunningOnionService>,
            OnionServiceRequestStream,
        ), error::ClientError> {
            let config = OnionServiceConfigBuilder::default()
            .nickname("arti-chat-service".parse()?)
            .build()?;

        let (onion_service, request_stream) = match client.launch_onion_service(config)? {
            Some(v) => v,
            None => {
                return Err(error::ClientError::OnionServiceDisabled);
            }
        };
        let request_stream = Box::new(request_stream);
        
        tracing::info!("Onion service launched.");
    
        Ok((onion_service, request_stream))
    }
}
