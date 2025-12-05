//! The client contains the logic to launch the Tor hidden service and encapsulates
//! services like the database, onion service,...

use arti_client::config::onion_service::OnionServiceConfigBuilder;
use crate::{db, error};
use ed25519_dalek::{SigningKey, VerifyingKey, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH};
use futures::{AsyncReadExt, Stream, StreamExt};
use tokio::sync::Mutex as TokioMutex;
use tor_cell::relaycell::msg::Connected;
use tor_proto::client::stream::IncomingStreamRequest;

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

    /// Private key of user to sign chat messages.
    private_key: SigningKey,

    /// Public key of user to verify received chat messages.
    public_key: VerifyingKey,
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

        // Generate new keypair.
        let (private_key, public_key) = Self::generate_keypair()?;

        // Store user with newly generated keypair.
        // Note that if the user with the given onion_id already exists
        // the user will not be inserted nor updated.
        let onion_id = Self::get_identity_unredacted_inner(onion_service.onion_address())?;
        let _ = db::UserDb {
            onion_id: onion_id.to_string(),
            nickname: "Me".to_string(),
            private_key,
            public_key,
        }.insert(db_conn.clone()).await;

        // Retrieve user again to get actual stored keypair.
        let user: db::UserDb = db::UserDb::retrieve(db_conn.clone(), &onion_id).await?;
        let (private_key, public_key) = Self::get_validated_keypair(&user.private_key, &user.public_key)?;

        tracing::info!("ArtiChat client launched.");
        Ok(Self {
            tor_client,
            onion_service,
            request_stream,
            private_key,
            public_key,
            db_conn,
        })
    }

    /// Main entrypoint/loop to accept requests from our hidden onion service.
    pub async fn serve(&self) -> Result<(), error::ClientError> {
        let mut request_stream = self.request_stream.lock().await;
        let requests = tor_hsservice::handle_rend_requests(&mut *request_stream);
        tokio::pin!(requests);

        while let Some(request) = requests.next().await {
            tokio::spawn(async move {
                let _ = Self::handle_request(request).await;
            });
        }

        Ok(())
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
    
    fn get_identity_unredacted_inner(onion_address: Option<arti_client::HsId>) -> Result<String, error::ClientError> {
        onion_address
            .ok_or(error::ClientError::EmptyHsid)
            .map(|address| safelog::DispUnredacted(address).to_string())
    }

    // Generate + store keypair to sign and verify chat messages.
    fn generate_keypair() -> Result<(String, String), error::ClientError> 
    {
        // Generate new keypair.
        let private_key = SigningKey::generate(&mut rand_core::OsRng);
        let public_key : [u8; PUBLIC_KEY_LENGTH] = private_key.verifying_key().to_bytes();
        let private_key : [u8; SECRET_KEY_LENGTH] = private_key.to_bytes();
        let public_key = hex::encode(public_key);
        let private_key = hex::encode(private_key);

        Ok((private_key, public_key))
    }

    // Get keypair from user and return if valid.
    fn get_validated_keypair(private_key: &str, public_key: &str) -> Result<(SigningKey, VerifyingKey), error::ClientError> {
        let private_key_b = hex::decode(private_key)?;
        let public_key_b = hex::decode(public_key)?;

        if private_key_b.len() != SECRET_KEY_LENGTH || public_key_b.len() != PUBLIC_KEY_LENGTH {
            return Err(error::ClientError::InvalidKeyLength);
        }

        let mut private_key = [0u8; SECRET_KEY_LENGTH];
        let mut public_key = [0u8; PUBLIC_KEY_LENGTH];
        private_key.copy_from_slice(&private_key_b);
        public_key.copy_from_slice(&public_key_b);

        Ok((
            SigningKey::from_bytes(&private_key),
            VerifyingKey::from_bytes(&public_key)?,
        ))
    }

    // Handle request from client to open new stream to our onion service.
    async fn handle_request(request: tor_hsservice::StreamRequest) -> Result<(), error::ClientError> {
        match request.request() {
            IncomingStreamRequest::Begin(begin) if begin.port() == 80 => {
                // Incoming request is a Begin message.

                // Accept request and get DataStream.
                let mut stream = request.accept(Connected::new_empty()).await?;

                // Read buffer.
                let mut read_buffer = std::vec::Vec::new();
                // We read stream byte-by-byte to handle bug in Arti with EndReason::Misc error.
                let mut byte = [0u8; 1];

                loop {
                    if let Ok(_) = stream.read_exact(&mut byte).await {
                        // We expect each message to end with a null-byte.
                        if byte[0] == 0 {
                            break;
                        }
                        read_buffer.push(byte[0]);
                    }
                }

                if read_buffer.is_empty() {
                    return Ok(());
                }

                let body = String::from_utf8_lossy(&read_buffer);

                tracing::debug!("Received request: {}", body);

                Ok(())
            },
            _ => {
                request.shutdown_circuit().expect("Failed to shutdown circuit.");
                Ok(())
            }
        }
    }
}
