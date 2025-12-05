import { useState } from "react";
import reactLogo from "./../assets/react.svg";
import { invoke } from "@tauri-apps/api/core";

import LeftNav from "./components/LeftNav/LeftNav";

import "./../styles/globals.scss";
import "./../App.scss";

function App() {
    return (
        <main className="container">
          <LeftNav />
        </main>
    );
}

export default App;
