import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./Loading.scss";

export default function Loading() {
    const [showRestart, setShowRestart] = useState(false);
    const [restarting, setRestarting] = useState(false);

    useEffect(() => {
        const timer = setTimeout(() => {
            setShowRestart(true);
        }, 20_000); // 20 seconds

        return () => clearTimeout(timer);
    }, []);

    const restartDaemon = async () => {
        try {
            setRestarting(true);
            await invoke("restart_daemon");
        } catch (err) {
            console.error("Failed to restart daemon", err);
        } finally {
            setRestarting(false);
        }
    };

    return (
        <div className="screen screen--loading">
            <span>Loading freedom...</span>

            {showRestart && (
                <button
                    className="loading__restart"
                    onClick={restartDaemon}
                    disabled={restarting}
                >
                    {restarting ? "Restarting…" : "Restart daemon"}
                </button>
            )}
        </div>
    );
}

