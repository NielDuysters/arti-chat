import { useState, useEffect } from "react";
import reactLogo from "./../assets/react.svg";
import { invoke } from "@tauri-apps/api/core";

import { Contact, useContacts } from "./hooks/useContacts";
import { useChat } from "./hooks/useChat";
import { useHiddenServicePing } from "./hooks/usePingHiddenService";
import { useDaemonPing } from "./hooks/usePingDaemon";

import Nav from "./components/Nav/Nav";
import ContactList from "./components/Contacts/ContactList";
import Loading from "./screens/Loading/Loading";
import Welcome from "./screens/Welcome/Welcome";
import AddContact from "./screens/AddContact/AddContact";
import ContactDetails from "./screens/ContactDetails/ContactDetails";
import UserDetails from "./screens/UserDetails/UserDetails";
import Settings from "./screens/Settings/Settings";
import TorCircuit from "./screens/TorCircuit/TorCircuit";
import Daemon from "./screens/Daemon/Daemon";
import ChatWindow from "./components/Chat/ChatWindow";

import "./../styles/globals.scss";
import "./../styles/screens.scss";
import "./../App.scss";

const App = () => {
    const [view, setView] = useState("welcome");
    const [activeContact, setActiveContact] = useState(null);
    const [contacts, setContacts] = useState<Contact[]>([]);
    const [initialLoadDone, setInitialLoadDone] = useState(false); 
   
    const { loadContacts } = useContacts({contacts: contacts, setContacts: setContacts});
    
    const { daemonIsReachable, setDaemonIsReachable } = useDaemonPing();
    const { hsIsReachable } = useHiddenServicePing();
    const {messages, sendMessage, sendAttachment } = useChat({activeContact: activeContact, loadContacts: loadContacts});

    // Load contacts once on mount.
    useEffect(() => {
        if (daemonIsReachable === true && !initialLoadDone) {
            setInitialLoadDone(true);
            loadContacts();
        }
    }, [daemonIsReachable]);

    // Set activeContact to null if going to different screen then chat.
    useEffect(() => {
        if (view !== "chat" && view !== "contact-details") {
            setActiveContact(null);
        }
    }, [view]);

    // Show loading screen if daemon is not active yet.
    if (!initialLoadDone) {
        return <Loading />;
    }

    const renderView = () => {
        switch (view) {
            case "welcome":
                return <Welcome />
            case "chat":
                return <ChatWindow
                            activeContact={activeContact}
                            loadContacts={loadContacts}
                            messages={messages}
                            sendMessage={sendMessage}
                            sendAttachment={sendAttachment}
                            setView={setView}
                        />
            case "add-contact":
                return <AddContact
                            contacts={contacts}
                            setContacts={setContacts}
                            setActiveContact={setActiveContact}
                            setView={setView}
                        />
            case "contact-details":
                return <ContactDetails
                            activeContact={activeContact}
                            contacts={contacts}
                            setContacts={setContacts}
                            setView={setView}
                        />
            case "user-details":
                return <UserDetails />
            case "settings":
                return <Settings
                            contacts={contacts}
                            setContacts={setContacts}
                        />
            case "tor-circuit":
                return <TorCircuit
                            hsIsReachable={hsIsReachable}
                        />
            case "daemon":
                return <Daemon
                            daemonIsReachable={daemonIsReachable}
                            setDaemonIsReachable={setDaemonIsReachable}
                        />
        }
    }

    return (
        <main className="container">
          <Nav
            setView={setView}
            hsIsReachable={hsIsReachable}
            daemonIsReachable={daemonIsReachable}
          />
          <ContactList
            contacts={contacts}
            setActiveContact={setActiveContact}
            setView={setView}
          />

          <div className="screen-container">
            {renderView()}
          </div>
        </main>
    );
}

export default App;
