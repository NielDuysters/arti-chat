# arti-chat-daemon

Library containing all the core logic:
- Connecting to Tor.
- Launching hidden onion service.
- Database.
- Message signing.
- Sending and receiving messages.
- RPC commands.
- ...

`arti-chat-daemon` communicates with the UI using RPC calls over UnixSockets. This library is called in the binary `arti-chat-daemon-bin` which daemonizes itself to the background, this way, messages can be received even if the UI is closed.
