# arti-chat-desktop-app

The GUI of the desktop app made with Tauri. The interface is built using React and the desktop app contains the code to use RPC calls to communicate with the daemon over unix sockets. The desktop app itself does not contain logic to send or receive messages, read from the database or connect to Tor. All that core logic can be found in `arti-chat-daemon`. We decided to separate the UI from the core logic, so messages can be received even when the UI is closed.

When a user clicks on the button to send e.g a message, the React/Javascript invokes a Tauri command. This Tauri command uses IPC (UnixSockets) to send a RPC call to the daemon. The daemon responds to that RPC call with the requested data or success state.
