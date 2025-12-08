import React from "react";
import "./Nav.scss";

export default function Nav() {
  return (
    <div class="nav">
      <div className="nav__item" data-screen="me">
        <img src="/src/assets/me.png" alt="ME" />
      </div>

      <div className="nav__item" data-screen="settings">
        <img src="/src/assets/settings.png" alt="Settings" />
      </div>

      <div className="nav__item" data-screen="daemon">
        <img src="/src/assets/daemon-active.png" alt="Daemon" />
      </div>

      <div className="nav__item" data-screen="tor">
        <img src="/src/assets/tor-active.png" alt="Tor" />
      </div>

      <div className="nav__item" data-screen="faq">
        <img src="/src/assets/faq.png" alt="FAQ" />
      </div>
    </div>
  );
}

