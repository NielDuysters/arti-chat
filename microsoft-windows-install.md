# Installation for Microsoft Windows

I added support for named pipes (alternative to UnixSockets for inter-process communication) and from now on Arti Chat can be used on Windows. The installation process is however not straightforward.

Sadly the (lack of) ergonomics on Windows make it hard to provide an automated installation script like we currently have for *nix systems. The only way to get Arti Chat to run on Windows is by cloning the git repository and manually build the source.
Unlike *nix, Windows does not come preinstalled with a few required developer tools like Perl, Python, Clang, C++ build essentials,... making a uniform installation procedure unfeasible - so I'll assume you already have these tools installed or understand how to install them.

During the development phase I will not make a dedicated installation script for Windows. In the future an app bundle for user-friendly installation will be provided.

## Prerequisites
- Rust + Cargo
- Tauri
- npm

## Build source

1. Clone the repository: `git clone https://github.com/NielDuysters/arti-chat.git`
2. Change directory `cd arti-chat/arti-chat-daemon-bin` and build `cargo build --release`.
    1. This will create a binary for the daemon in the target folder. You can move it to e.g. `C:\Program Files\Arti Chat\`.
3. Change directory `cd ../arti-chat/arti-chat-desktop-app` and build `cargo tauri build`.
    1. Execute the MSI file from the output to install Arti Chat.

## Daemonize arti-chat-daemon

Arti Chat uses a background service so you can receive messages and notifications even when you have the app closed.

For the time being, the easiest way to achieve this is by using [Servy](https://servy-win.github.io/).

<img width="715" height="863" alt="servy" src="https://github.com/user-attachments/assets/c91a2a3e-bf44-44ae-871b-2219abc09dc9" />

Install and start the service to make the daemon run in the background.

## Roadmap

When Arti Chat gets released, we will of course provide a more user-friendly way for users to install. :)
