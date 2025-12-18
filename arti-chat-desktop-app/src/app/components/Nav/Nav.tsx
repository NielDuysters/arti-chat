import React from "react";
import "./Nav.scss";

export default function Nav({setView, hsIsReachable, daemonIsReachable}) {
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
                <img src="/assets/me.png" alt="ME" />
            </div>

            <div
                className="nav__item"
                onClick={() => {
                    setView("settings");
                }}
            >
                <img src="/assets/settings.png" alt="Settings" />
            </div>

            <div
                className="nav__item"
                onClick={() => {
                    setView("daemon");
                }}
            >
                <img src="/assets/daemon.png" alt="Daemon" />
                {statusIndicator(daemonIsReachable)}
            </div>

            <div
                className="nav__item"
                onClick={() => {
                    setView("tor-circuit");
                }}
            >
                <img src="/assets/tor-circuit.png" alt="Tor" />
                {statusIndicator(hsIsReachable)}
            </div>

            <div className="nav__item" data-screen="faq">
                <img src="/assets/faq.png" alt="FAQ" />
            </div>
        </div>
    );
}

