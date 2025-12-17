import React, { useEffect, useState } from "react";
import { useClient } from "../../hooks/useClient";

import "./Loading.scss";

export default function Loading() {
    const [showRestart, setShowRestart] = useState(false);
    const [restarting, setRestarting] = useState(false);
    const { restartDaemon } = useClient();

    useEffect(() => {
        const timer = setTimeout(() => {
            setShowRestart(true);
        }, 25000);

        return () => clearTimeout(timer);
    }, []);

    const restart = async () => {
        try {
            setRestarting(true);
            await restartDaemon();
        } catch (err) {
            console.error("Failed to restart daemon", err);
        } finally {
            setRestarting(false);
        }
    };

    return (
        <div className="screen screen--loading">
            <span className="screen--loading__title">Loading free speech...</span>

            {showRestart && (
                <div>
                    <p>The daemon is offline, try restarting it...</p>
                    <div
                        className="screen--loading__restart-btn"
                        onClick={restart}
                        disabled={restarting}
                    >
                        {restarting ? "Restarting…" : "Restart daemon"}
                    </div>
                </div>
            )}
        </div>
    );
}

