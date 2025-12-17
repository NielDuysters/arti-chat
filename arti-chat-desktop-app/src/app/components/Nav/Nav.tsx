import React from "react";
import "./Nav.scss";

export default function Nav({setView, isReachable}) {
    const statusIndicator = (status) => {
        if (status === null) {
            return;
        }

        return (
            <div className={`nav__item__status ${status ? "nav__item__status--ok" : "nav__item__status--nok"}`}></div>
        );
    }

    return (
        <div className="nav">
            <div
                className="nav__item"
                onClick={() => {
                    setView("user-details");
                }}
            >
                <img src="/src/assets/me.png" alt="ME" />
            </div>

            <div
                className="nav__item"
                onClick={() => {
                    setView("settings");
                }}
            >
                <img src="/src/assets/settings.png" alt="Settings" />
            </div>

            <div className="nav__item" data-screen="daemon">
                <img src="/src/assets/daemon-active.png" alt="Daemon" />
            </div>

            <div
                className="nav__item"
                onClick={() => {
                    setView("tor-circuit");
                }}
            >
                <img src="/src/assets/tor-circuit.png" alt="Tor" />
                {statusIndicator(isReachable)}
            </div>

            <div className="nav__item" data-screen="faq">
                <img src="/src/assets/faq.png" alt="FAQ" />
            </div>
        </div>
    );
}

