import { useState } from "react";
import reactLogo from "./../assets/react.svg";
import { invoke } from "@tauri-apps/api/core";

import Nav from "./components/Nav/Nav";
import ContactList from "./components/Contacts/ContactList";

import "./../styles/globals.scss";
import "./../App.scss";

function App() {
    return (
        <main className="container">
          <Nav />
          <ContactList />
        </main>
    );
}

export default App;
