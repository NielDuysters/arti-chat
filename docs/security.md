# Security goals

Arti Chat currently already contains implementations with the intent to make it a secure, private and anonymous messaging app. But in this stage we cannot be sure these implementations are solid. This document lists the current attempts to make Arti Chat secure -- **which need auditing**!

Feel free to open an issue to discuss one or more points.

- [ ] The code is safe and does not contain vulnerabilities like buffer overflow.
- [ ] IP-addresses of users are hidden since all traffic is routed through Tor.
- [ ] Conversations are end-to-end encrypted (single ratched) meaning that past messages can't be decrypted when a user's key is compromised.
- [ ] Incoming and outgoing messages are private thanks to Tor's encryption + our own e2ee.
- [ ] Messages do not contain metadata.
- [ ] Sent images do not contain metadata.
- [ ] Users can't be deaonymized by their hidden onion service.
- [ ] Each user has a keypair decoupled from his Tor identity so he can't be imitated by an adversary.
- [ ] Sqlcipher encrypts the local database so the user is protected against data theft, unless the user's keyring is compromised.

