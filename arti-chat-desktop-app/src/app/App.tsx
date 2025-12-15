import { useState, useEffect } from "react";
import reactLogo from "./../assets/react.svg";
import { invoke } from "@tauri-apps/api/core";

import { Contact, useContacts } from "./hooks/useContacts";

import Nav from "./components/Nav/Nav";
import ContactList from "./components/Contacts/ContactList";
import Welcome from "./screens/Welcome/Welcome";
import AddContact from "./screens/AddContact/AddContact";
import ContactDetails from "./screens/ContactDetails/ContactDetails";
import UserDetails from "./screens/UserDetails/UserDetails";
import ChatWindow from "./components/Chat/ChatWindow";

import "./../styles/globals.scss";
import "./../styles/screens.scss";
import "./../App.scss";

const App = () => {
    const [view, setView] = useState("welcome");
    const [activeContact, setActiveContact] = useState(null);
    const [contacts, setContacts] = useState<Contact[]>([]);
   
    const { loadContacts } = useContacts({contacts: contacts, setContacts: setContacts});

    // Load contacts once on mount.
    useEffect(() => {
        loadContacts();
    }, []);

    const renderView = () => {
        switch (view) {
            case "welcome":
                return <Welcome />
            case "chat":
                return <ChatWindow activeContact={activeContact} setView={setView} />
            case "add-contact":
                return <AddContact
                            contacts={contacts}
                            setContacts={setContacts}
                        />
            case "contact-details":
                return <ContactDetails
                            activeContact={activeContact}
                            contacts={contacts}
                            setContacts={setContacts}
                        />
            case "user-details":
                return <UserDetails />
        }
    }


    return (
        <main className="container">
          <Nav setView={setView} />
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
