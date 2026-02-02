# Review notes

This project is still a draft and far from finalized but feedback on the following topics would already be very welcome!

## Trivial questions

- What part of the current setup worries you most?
- If you could fix one thing in this, what would it be?
- Where would you try to attack this app if you were an advesary? Where do you suspect there are vulnerabilities?

## Rust code

- Do you see any anti-patterns in the code?
- Am I currently applying bad habits in my code?
- Is my code readable?
- Am I handling async and threading correctly?
- How is my error handling?

## Project file structure

- Is my Rust code structured logically?
- Are the names of the modules and files clear?
- How would you structure it differently? More modules? More subdirectories?

## Architecture

This project contains three parts:
- arti-chat-daemon: Library containing all the core logic (database, cryptography, networking,...)
- arti-chat-daemon-bin: Small binary calling arti-chat-daemon, this bin is served as a daemon.
- arti-chat-desktop-app: GUI

The GUI communicates with the core using IPC.

- Are there any disadvantages to the architecture?
- How could it be improved?
- Would you've done it differently?

## Tor / networking / streams

- Am I using the Arti Tor Client correctly?
- Do you see any inefficiencies in how streams, channels, MPSC, sockets,... are handled?
- Is it possible I am misunderstanding networking basics and have wrong implementations?

## End-to-End Encryption

- Does my e2ee implementation look solid on first sight (single ratchet)? (I'll let this audit later btw)
- Am I understanding the theory behind the cryptograpy correct?

## Security

- Do you notice obvious security issues which could result in an attack or deanonymization?
