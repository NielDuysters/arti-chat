//! The client contains the logic to launch the Tor hidden service and encapsulates
//! services like the database, onion service,...

use crate::{
    db::{self, DbModel, DbUpdateModel},
    error,
    ipc::{self, MessageToUI},
    message, ui_focus, PROJECT_DIR,
};
use arti_client::config::onion_service::OnionServiceConfigBuilder;
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SigningKey, VerifyingKey};
use futures::{AsyncReadExt, AsyncWriteExt, Stream, StreamExt};
use notify_rust::Notification;
use tokio::sync::Mutex as TokioMutex;
use tokio::sync::mpsc::UnboundedSender;
use tor_cell::relaycell::msg::Connected;
use tor_proto::client::stream::IncomingStreamRequest;

use crate::session;
use std::collections::HashMap;

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

    sessions: std::sync::Arc<TokioMutex<HashMap<String, session::Session>>>,
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
            enable_notifications: db::ConfigDb::get_bool("enable_notifications", db_conn.clone())
                .await?,
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
    async fn ensure_session(
        &self,
        peer_onion: &str,
    ) -> Result<tokio::sync::MutexGuard<'_, HashMap<String, session::Session>>, error::ClientError> {
        let my_onion = self.get_identity_unredacted()?;
        let mut sessions = self.sessions.lock().await;

        if sessions.contains_key(peer_onion) {
            return Ok(sessions);
        }

        // --- perform handshake ---
        let contact = db::ContactDb::retrieve(peer_onion, self.db_conn.clone()).await?;
        let peer_verify = session::verifying_key_from_hex(&contact.public_key)?;

        let (init, my_eph) =
            session::initiate_handshake(&my_onion, peer_onion, &self.private_key);

        let mut stream = self
            .tor_client
            .lock()
            .await
            .connect(&format!("{peer_onion}:80"))
            .await?;

        let mut out = serde_json::to_string(&init)?;
        out.push('\0');
        stream.write_all(out.as_bytes()).await?;

        let reply_raw = session::read_null_terminated(&mut stream).await?;
        let reply: session::Handshake = serde_json::from_str(&reply_raw)?;

        let sess = session::complete_handshake(
            &reply,
            &my_onion,
            &peer_verify,
            my_eph,
            true, // initiator
        )?;

        sessions.insert(peer_onion.to_string(), sess);
        Ok(sessions)
    }


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
        }
        .insert(db_conn.clone())
        .await;

        // Retrieve user again to get actual stored keypair.
        let user: db::UserDb = db::UserDb::retrieve(&onion_id, db_conn.clone()).await?;
        let (private_key, _public_key) =
            Self::get_validated_keypair(&user.private_key, &user.public_key)?;

        tracing::info!("ArtiChat client launched.");
        Ok(Self {
            tor_client: TokioMutex::new(tor_client),
            db_conn: db_conn.clone(),
            config: std::sync::Arc::new(TokioMutex::new(
                ClientConfig::load(db_conn.clone()).await?,
            )),
            onion_service,
            request_stream,
            private_key,
            sessions: std::sync::Arc::new(TokioMutex::new(HashMap::new())),
        })
    }

    /// Main entrypoint/loop to accept requests from our hidden onion service.
    /*
    pub async fn serve(
        &self,
        message_tx: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> Result<(), error::ClientError> {
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
    }*/

    pub async fn serve(
        &self,
        message_tx: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> Result<(), error::ClientError> {
        let mut request_stream = self.request_stream.lock().await;
        let requests = tor_hsservice::handle_rend_requests(&mut *request_stream);
        tokio::pin!(requests);

        // Capture shared state once
        let sessions = self.sessions.clone();
        let signing_key = self.private_key.clone();
        let my_onion_id = self.get_identity_unredacted()?;

        while let Some(request) = requests.next().await {
            let message_tx = message_tx.clone();
            let db_conn = self.db_conn.clone();
            let client_config = self.config.clone();

            let sessions = sessions.clone();
            let signing_key = signing_key.clone();
            let my_onion_id = my_onion_id.clone();

            tokio::spawn(async move {
                let _ = Client::handle_request(
                    request,
                    message_tx,
                    db_conn,
                    client_config,
                    my_onion_id,
                    signing_key,
                    sessions,
                )
                .await;
            });
        }

        Ok(())
    }


    /// Send message to peer.
    /*
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
    }*/

    pub async fn send_message_to_peer(
        &self,
        to_onion_id: &str,
        text: &str,
    ) -> Result<(), error::ClientError> {
        let my_onion = self.get_identity_unredacted()?;

        // 1) ensure session
        let mut sessions = self.ensure_session(to_onion_id).await?;
        let session = sessions.get_mut(to_onion_id).unwrap();

        // 2) encrypt
        #[derive(serde::Serialize)]
        struct Plaintext {
            onion_id: String,
            text: String,
            timestamp: i64,
        }

        let plain = Plaintext {
            onion_id: my_onion.clone(),
            text: text.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        let encrypted = session::encrypt(
            session,
            &serde_json::to_vec(&plain)?,
            my_onion,
        );

        drop(sessions); // release lock before network I/O

        // 3) send
        let mut stream = self
            .tor_client
            .lock()
            .await
            .connect(&format!("{to_onion_id}:80"))
        .await?;

        let mut msg = serde_json::to_string(&encrypted)?;
        msg.push('\0');
        stream.write_all(msg.as_bytes()).await?;

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
                    .send_message_to_peer(&msg.contact_onion_id, &msg.body)
                    .await;

                if retry.is_ok() {
                    tracing::info!("Retry success message {}", msg.id);

                    // On success update sent_status + update UI.
                    db::UpdateMessageDb {
                        id: msg.id,
                        sent_status: Some(true),
                    }
                    .update(self.db_conn.clone())
                    .await?;

                    #[derive(serde::Serialize)]
                    struct SendIncomingMessage {
                        /// HsId from peer we received this message from.
                        pub onion_id: String,
                    }
                    let incoming_message = SendIncomingMessage {
                        onion_id: msg.contact_onion_id.to_string(),
                    };
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
            Err(_) => Ok(false),
            Ok(Ok(_stream)) => Ok(true),
            Ok(Err(_)) => Ok(false),
        }
    }

    /// Bootstrap connection to Tor network and return client.
    async fn bootstrap_tor_client() -> Result<ArtiTorClient, error::ClientError> {
        let state_dir = PROJECT_DIR.data_local_dir().join("hsstate");
        let cache_dir = state_dir.join("cache");
        let config = arti_client::config::TorClientConfigBuilder::
            from_directories(
                state_dir,
                cache_dir
            )
        .build()?;
        let client = arti_client::TorClient::create_bootstrapped(config).await?;

        tracing::info!("Tor Client bootstrapped.");

        Ok(client)
    }

    /// Launch our hidden service.
    async fn launch_onion_service(
        client: &ArtiTorClient,
    ) -> Result<
        (
            std::sync::Arc<tor_hsservice::RunningOnionService>,
            OnionServiceRequestStream,
        ),
        error::ClientError,
    > {
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
    fn get_identity_unredacted_inner(
        onion_address: Option<arti_client::HsId>,
    ) -> Result<String, error::ClientError> {
        onion_address
            .ok_or(error::ClientError::EmptyHsid)
            .map(|address| safelog::DispUnredacted(address).to_string())
    }

    /// Generate + store keypair to sign and verify chat messages.
    fn generate_keypair() -> (String, String) {
        // Generate new keypair.
        let private_key = SigningKey::generate(&mut rand_core::OsRng);
        let public_key: [u8; PUBLIC_KEY_LENGTH] = private_key.verifying_key().to_bytes();
        let private_key: [u8; SECRET_KEY_LENGTH] = private_key.to_bytes();
        let public_key = hex::encode(public_key);
        let private_key = hex::encode(private_key);

        (private_key, public_key)
    }

    /// Get keypair from user and return if valid.
    fn get_validated_keypair(
        private_key: &str,
        public_key: &str,
    ) -> Result<(SigningKey, VerifyingKey), error::ClientError> {
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
    /*
    async fn handle_request(
        request: tor_hsservice::StreamRequest,
        message_tx: tokio::sync::mpsc::UnboundedSender<String>, // Used to send incoming messages
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
                let contact_public_key =
                    db::ContactDb::retrieve(&payload.onion_id, db_conn.clone())
                        .await?
                        .public_key;
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
                }
                .insert(db_conn.clone())
                .await?;

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
            }
            _ => {
                let _ = request.shutdown_circuit();
                Ok(())
            }
        }
    }*/

    async fn handle_request(
        request: tor_hsservice::StreamRequest,
        message_tx: tokio::sync::mpsc::UnboundedSender<String>,
        db_conn: DatabaseConnection,
        client_config: ClientConfigType,
        my_onion_id: String,
        signing_key: SigningKey,
        sessions: std::sync::Arc<tokio::sync::Mutex<
        std::collections::HashMap<String, session::Session>,
        >>,
    ) -> Result<(), error::ClientError> {
        match request.request() {
            IncomingStreamRequest::Begin(begin) if begin.port() == 80 => {
                // Accept incoming stream
                let mut stream = request.accept(Connected::new_empty()).await?;

                // Read exactly one framed message
                let body = session::read_null_terminated(&mut stream).await?;
                if body.is_empty() {
                    return Ok(());
                }

                tracing::debug!("Incoming: {}", body);

                // First try handshake
                if let Ok(handshake) = serde_json::from_str::<session::Handshake>(&body) {
                    // --- HANDSHAKE PATH ---

                    // Must be addressed to us
                    if handshake.to != my_onion_id {
                        return Ok(());
                    }

                    // Lookup sender identity key
                    let contact = db::ContactDb::retrieve(&handshake.from, db_conn.clone()).await?;
                    let peer_verify =
                    session::verifying_key_from_hex(&contact.public_key)?;

                    // Accept handshake
                    let (reply, my_eph) =
                    session::accept_handshake(&handshake, &my_onion_id, &peer_verify, &signing_key)?;

                    // Send reply
                    let mut out = serde_json::to_string(&reply)?;
                    out.push('\0');
                    stream.write_all(out.as_bytes()).await?;
                    stream.flush().await.ok();

                    // Finalize session (responder side)
                    let sess = session::complete_handshake(
                        &reply,
                        &my_onion_id,
                        &peer_verify,
                        my_eph,
                        false, // responder
                    )?;

                    // Store session
                    let mut sessions = sessions.lock().await;
                    sessions.insert(handshake.from.clone(), sess);

                    tracing::info!(
                        "Session established with {}",
                        handshake.from
                    );

                    return Ok(());
                }

                // Otherwise it must be encrypted data
                let encrypted: session::Encrypted = serde_json::from_str(&body)?;

                // --- ENCRYPTED MESSAGE PATH ---

                // Load session
                let mut sessions_guard = sessions.lock().await;
                let session = sessions_guard
                    .get_mut(&encrypted.from_onion_id)
                    .ok_or_else(|| {
                        error::ClientError::ArtiBug
                    })?;

                // Decrypt (handles out-of-order internally)
                let plaintext =
                session::decrypt(session, &encrypted)?;

                drop(sessions_guard); // release lock early

                // Parse decrypted payload
                #[derive(serde::Deserialize, serde::Serialize)]
                struct PlaintextPayload {
                    onion_id: String,
                    text: String,
                    timestamp: i64,
                }

                let payload: PlaintextPayload =
                serde_json::from_slice(&plaintext)?;

                // Store message
                db::MessageDb {
                    id: 0,
                    contact_onion_id: payload.onion_id.clone(),
                    body: payload.text.clone(),
                    timestamp: payload.timestamp as i32,
                    is_incoming: true,
                    sent_status: false,
                    verified_status: true, // session-authenticated
                }
                    .insert(db_conn.clone())
                .await?;

                // Send to UI
                let _ = message_tx.send(serde_json::to_string(&payload)?);

                // Notification
                let cfg = client_config.lock().await;
                if cfg.enable_notifications && !ui_focus::is_focussed() {
                    let _ = notify_rust::Notification::new()
                        .summary("Arti Chat")
                        .body("New message received")
                        .show();
                }

                Ok(())
            }

            _ => {
                let _ = request.shutdown_circuit();
                Ok(())
            }
        }
    }


}
