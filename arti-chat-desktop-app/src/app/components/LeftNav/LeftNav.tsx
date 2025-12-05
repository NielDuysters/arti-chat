import React from "react";
import "./LeftNav.scss";

export default function LeftNav() {
  return (
    <div id="left-nav-wrapper">
      <div className="left-nav-option" data-screen="me">
        <img src="/src/assets/me.png" alt="ME" />
      </div>

      <div className="left-nav-option" data-screen="settings">
        <img src="/src/assets/settings.png" alt="Settings" />
      </div>

      <div className="left-nav-option" data-screen="daemon">
        <img src="/src/assets/daemon-active.png" alt="Daemon" />
      </div>

      <div className="left-nav-option" data-screen="tor">
        <img src="/src/assets/tor-active.png" alt="Tor" />
      </div>

      <div className="left-nav-option" data-screen="faq">
        <img src="/src/assets/faq.png" alt="FAQ" />
      </div>
    </div>
  );
}

