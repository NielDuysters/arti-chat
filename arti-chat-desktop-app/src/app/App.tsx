import { useState, useEffect } from "react";
import reactLogo from "./../assets/react.svg";
import { invoke } from "@tauri-apps/api/core";

import { useContacts } from "./hooks/useContacts";

import Nav from "./components/Nav/Nav";
import ContactList from "./components/Contacts/ContactList";
import Welcome from "./screens/Welcome/Welcome";
import AddContact from "./screens/AddContact/AddContact";
import ChatWindow from "./components/Chat/ChatWindow";

import "./../styles/globals.scss";
import "./../styles/screens.scss";
import "./../App.scss";

const App = () => {
    const [view, setView] = useState("welcome");
    const [activeContact, setActiveContact] = useState(null);

    // Load contacts on startup.
    const { contacts } = useContacts();

    const renderView = () => {
        switch (view) {
            case "welcome":
                return <Welcome />
            case "chat":
                return <ChatWindow activeContact={activeContact} />
            case "add-contact":
                return <AddContact />
        }
    }


    return (
        <main className="container">
          <Nav />
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
