import { useState } from "react";
import reactLogo from "./../assets/react.svg";
import { invoke } from "@tauri-apps/api/core";

import LeftNav from "./components/LeftNav/LeftNav";
import ContactList from "./components/ContactList/ContactList";

import "./../styles/globals.scss";
import "./../App.scss";

function App() {
    return (
        <main className="container">
          <LeftNav />
          <ContactList />
        </main>
    );
}

export default App;
