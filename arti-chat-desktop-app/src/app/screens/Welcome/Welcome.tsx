import React from "react";
import "./Welcome.scss";

export default function Welcome()  {
    return (
        <div className="screen screen--welcome">
            <h2>Welcome to ArtiChat</h2>
            <h4>You can talk freely now.</h4>
            <a className="screen--welcome__donate-btn" href="https://donate.torproject.org" target="_blank">
                <span>Donate to Tor</span>
            </a>
        </div>
    );
}
