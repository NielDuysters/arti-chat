//! The client contains the logic to launch the Tor hidden service and encapsulates
//! services like the database, onion service,...

use arti_client::config::onion_service::OnionServiceConfigBuilder;
use crate::{db::{self, DbModel, DbUpdateModel}, error, ipc::{self, MessageToUI}, message, ui_focus};
use ed25519_dalek::{SigningKey, VerifyingKey, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH};
use futures::{AsyncReadExt, AsyncWriteExt, Stream, StreamExt};
use notify_rust::Notification;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex as TokioMutex;
use tor_cell::relaycell::msg::Connected;
use tor_proto::client::stream::IncomingStreamRequest;

/// Type for TorClient with runtime.
type ArtiTorClient = arti_client::TorClient<tor_rtcompat::PreferredRuntime>;
/// Type for hidden service request stream behind boxed smart pointer.
type OnionServiceRequestStream = Box<dyn Stream<Item = tor_hsservice::RendRequest> + Send>;
/// Type for thread-safe database connection.
type DatabaseConnection = std::sync::Arc<TokioMutex<rusqlite::Connection>>; 
/// Type for thread-safe ClientConfig.
type ClientConfigType = std::sync::Arc<TokioMutex<ClientConfig>>; 

/// Encapsulates hidden service, database connection,...
pub struct Client {
    /// Arti Tor Client.
    pub tor_client: TokioMutex<ArtiTorClient>,

    /// Database connection.
    pub db_conn: DatabaseConnection,

    /// Configuration from database.
    pub config: ClientConfigType,

    /// Running hidden onion service.
    onion_service: std::sync::Arc<tor_hsservice::RunningOnionService>,

    /// Request stream of onion service to handle incoming requests.
    /// We have to store it seperately because we can't derive it after `launch_onion_service`.
    request_stream: TokioMutex<std::pin::Pin<OnionServiceRequestStream>>,

    /// Private key of user to sign chat messages.
    private_key: SigningKey,
}

/// Client configuration from database.
#[non_exhaustive]
pub struct ClientConfig {
    /// Enable/disable notifications.
    pub enable_notifications: bool,
}

impl ClientConfig {
    /// (Re)load client configuration from database.
    pub async fn load(db_conn: DatabaseConnection) -> Result<Self, error::ClientError> {
        Ok(Self {
            enable_notifications: db::ConfigDb::get_bool("enable_notifications", db_conn.clone()).await?,
        })
    }

    /// Get config value.
    pub fn get(&self, key: &ClientConfigKey) -> String {
        match key {
            ClientConfigKey::EnableNotifications => self.enable_notifications.to_string(),
        }
    }
}

/// Possible config keys.
#[non_exhaustive]
pub enum ClientConfigKey {
    /// Setting for desktop notifications for incoming messages.
    EnableNotifications,
}

impl std::str::FromStr for ClientConfigKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "enable_notifications" => Ok(Self::EnableNotifications),
            _ => Err(()),
        }
    }
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
        let (private_key, public_key) = Self::generate_keypair();

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
        let user: db::UserDb = db::UserDb::retrieve(&onion_id, db_conn.clone()).await?;
        let (private_key, _public_key) = Self::get_validated_keypair(&user.private_key, &user.public_key)?;

        tracing::info!("ArtiChat client launched.");
        Ok(Self {
            tor_client: TokioMutex::new(tor_client),
            db_conn: db_conn.clone(),
            config: std::sync::Arc::new(TokioMutex::new(ClientConfig::load(db_conn.clone()).await?)),
            onion_service,
            request_stream,
            private_key,
        })
    }

    /// Main entrypoint/loop to accept requests from our hidden onion service.
    pub async fn serve(&self, message_tx: tokio::sync::mpsc::UnboundedSender<String>) -> Result<(), error::ClientError> {
        let mut request_stream = self.request_stream.lock().await;
        let requests = tor_hsservice::handle_rend_requests(&mut *request_stream);
        tokio::pin!(requests);

        while let Some(request) = requests.next().await {
            let message_tx = message_tx.clone();
            let db_conn = self.db_conn.clone();
            let client_config = self.config.clone();
            tokio::spawn(async move {
                let _ = Self::handle_request(request, message_tx, db_conn, client_config).await;
            });
        }

        Ok(())
    }

    /// Send message to peer.
    pub async fn send_message_to_peer(
        &self,
        to_onion_id: &str,
        text: &str, 
    ) -> Result<(), error::ClientError> {
        // Open stream to peer.
        let target = format!("{}:80", to_onion_id);
        let tor_client = self.tor_client.lock().await;
        let mut stream = tor_client.connect(&target).await?;

        // Make payload.
        let payload = message::MessagePayload {
            onion_id: self.get_identity_unredacted()?,
            text: text.into(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        // Sign payload.
        let signed_payload = payload.sign_message(&mut self.private_key.clone())?;
        let mut signed_payload = serde_json::to_string(&signed_payload)?;
        signed_payload.push('\0'); // Null-byte to signify ending of stream.

        // Send payload over stream to peer.
        stream.write_all(signed_payload.as_bytes()).await?;
        stream.flush().await.ok();

        Ok(())
    }

    /// Retry sending failed messages.
    pub async fn retry_failed_messages(
        &self,
        broadcast_writers: std::sync::Arc<TokioMutex<Vec<UnboundedSender<ipc::MessageToUI>>>>,
    ) -> Result<(), error::ClientError> {
        loop {
            let failed_messages = db::MessageDb::failed_messages(self.db_conn.clone()).await?;
            for msg in &failed_messages {
                tracing::info!("Retrying message {}", msg.id);

                // Retry sending.
                let retry = self
                    .send_message_to_peer(&msg.contact_onion_id, &msg.body).await;

                if retry.is_ok() {
                    tracing::info!("Retry success message {}", msg.id);

                    // On success update sent_status + update UI.
                    db::UpdateMessageDb {
                        id: msg.id,
                        sent_status: Some(true),
                    }.update(self.db_conn.clone()).await?;

                    #[derive(serde::Serialize)]
                    struct SendIncomingMessage {
                        /// HsId from peer we received this message from.
                        pub onion_id: String,
                    }
                    let incoming_message = SendIncomingMessage { onion_id: msg.contact_onion_id.to_string() };
                    let incoming_message = serde_json::to_string(&incoming_message)? + "\n";
                    let bw_writers = broadcast_writers.lock().await;
                    for tx in bw_writers.iter() {
                        let _ = tx.send(MessageToUI::Broadcast(incoming_message.clone()));
                    }
                } else {
                    tracing::info!("Retry failed message {}", msg.id);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
        }
    }

    /// Get onion service identity unredacted.
    /// Warning: This displays the full hidden service onion url.
    pub fn get_identity_unredacted(&self) -> Result<String, error::ClientError> {
        self.onion_service
            .onion_address()
            .ok_or(error::ClientError::EmptyHsid)
            .map(|address| safelog::DispUnredacted(address).to_string())
    }

    /// Reset TorClient to connect over new circuit.
    pub async fn reset_tor_circuit(&self) -> Result<(), error::ClientError> {
        let mut prefs = arti_client::StreamPrefs::new();
        prefs.new_isolation_group();

        let mut tor_client = self.tor_client.lock().await;
        *tor_client = tor_client.clone_with_prefs(prefs);

        Ok(())
    }

    /// Reload configuration from database.
    pub async fn reload_config(&self) -> Result<(), error::ClientError> {
        let new_config = ClientConfig::load(self.db_conn.clone()).await?;
        let mut cfg = self.config.lock().await;
        *cfg = new_config;
        Ok(())
    }

    /// Checks if our hidden onion service is reachable.
    pub async fn is_reachable(&self) -> Result<bool, error::ClientError> {
        let onion_id = self.get_identity_unredacted()?;
        let target = format!("{}:80", onion_id);
        let tor_client = self.tor_client.lock().await;
        let connect_future = tor_client.connect(target);
        let result = tokio::time::timeout(std::time::Duration::from_secs(45), connect_future).await;

        match result {
            Err(_) => {
                Ok(false)
            },
            Ok(Ok(_stream)) => {
                Ok(true)
            }
            Ok(Err(_)) => {
                Ok(false)
            }
        }
    }

    /// Bootstrap connection to Tor network and return client.
    async fn bootstrap_tor_client() -> Result<ArtiTorClient, error::ClientError> {
        let config = arti_client::TorClientConfig::default();
        let client = arti_client::TorClient::create_bootstrapped(config).await?;

        tracing::info!("Tor Client bootstrapped.");

        Ok(client)
    }

    /// Launch our hidden service.
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

    /// Show HsId of hidden service.
    fn get_identity_unredacted_inner(onion_address: Option<arti_client::HsId>) -> Result<String, error::ClientError> {
        onion_address
            .ok_or(error::ClientError::EmptyHsid)
            .map(|address| safelog::DispUnredacted(address).to_string())
    }

    /// Generate + store keypair to sign and verify chat messages.
    fn generate_keypair() -> (String, String) 
    {
        // Generate new keypair.
        let private_key = SigningKey::generate(&mut rand_core::OsRng);
        let public_key : [u8; PUBLIC_KEY_LENGTH] = private_key.verifying_key().to_bytes();
        let private_key : [u8; SECRET_KEY_LENGTH] = private_key.to_bytes();
        let public_key = hex::encode(public_key);
        let private_key = hex::encode(private_key);

        (private_key, public_key)
    }

    /// Get keypair from user and return if valid.
    fn get_validated_keypair(private_key: &str, public_key: &str) -> Result<(SigningKey, VerifyingKey), error::ClientError> {
        let private_key_b = hex::decode(private_key)?;
        let public_key_b = hex::decode(public_key)?;

        if private_key_b.len() != SECRET_KEY_LENGTH || public_key_b.len() != PUBLIC_KEY_LENGTH {
            return Err(error::ClientError::InvalidKeyLength);
        }

        let mut private_key = [0_u8; SECRET_KEY_LENGTH];
        let mut public_key = [0_u8; PUBLIC_KEY_LENGTH];
        private_key.copy_from_slice(&private_key_b);
        public_key.copy_from_slice(&public_key_b);

        Ok((
            SigningKey::from_bytes(&private_key),
            VerifyingKey::from_bytes(&public_key)?,
        ))
    }

    /// Handle request from client to open new stream to our onion service.
    async fn handle_request(
        request: tor_hsservice::StreamRequest,
        message_tx: tokio::sync::mpsc::UnboundedSender<String>,  // Used to send incoming messages
        // to IPC server.
        db_conn: DatabaseConnection,
        client_config: ClientConfigType,
    ) -> Result<(), error::ClientError> {
        match request.request() {
            IncomingStreamRequest::Begin(begin) if begin.port() == 80 => {
                // Incoming request is a Begin message.

                // Accept request and get DataStream.
                let mut stream = request.accept(Connected::new_empty()).await?;

                // Read buffer.
                let mut read_buffer = std::vec::Vec::new();
                // We read stream byte-by-byte to handle bug in Arti with EndReason::Misc error.
                let mut byte = [0_u8; 1];

                while stream.read_exact(&mut byte).await.is_ok() {
                    // We expect each message to end with a null-byte.
                    if byte[0] == 0 {
                        break;
                    }
                    read_buffer.push(byte[0]);
                }

                if read_buffer.is_empty() {
                    return Ok(());
                }

                let body = String::from_utf8_lossy(&read_buffer);
                tracing::debug!("Received request: {}", body);

                // Verify incoming signed payload.
                let signed_payload: message::SignedMessagePayload = serde_json::from_str(&body)?;
                let payload = &signed_payload.payload;
                let contact_public_key = db::ContactDb::retrieve(&payload.onion_id, db_conn.clone()).await?.public_key;
                let verified = signed_payload.verify_message(&contact_public_key).is_ok();

                // Store incoming message in db.
                db::MessageDb {
                    id: 0,
                    contact_onion_id: payload.onion_id.to_string(),
                    body: payload.text.to_string(),
                    timestamp: payload.timestamp as i32,
                    is_incoming: true,
                    sent_status: false,
                    verified_status: verified,
                }.insert(db_conn.clone()).await?;

                // Send to message channel.
                let _ = message_tx.send(serde_json::to_string(payload)?);

                // Show notifcation for new message if user
                // is not actively using the app.
                let client_config = client_config.lock().await;
                if client_config.enable_notifications && !ui_focus::is_focussed() {
                    let _ = Notification::new()
                        .summary("Arti chat")
                        .body("You received a new message.")
                        .show();
                }

                Ok(())
            },
            _ => {
                let _ = request.shutdown_circuit();
                Ok(())
            }
        }
    }
}
