import React from "react";
import { useState, useEffect } from "react";
import { useClient } from "../../hooks/useClient";
import Action from "../../components/Action/Action";
import { ActionType } from "../../components/Action/ActionType";

export default function Daemon({daemonIsReachable})  {
    const { restartDaemon } = useClient();
    const [restartDaemonSuccess, setRestartDaemonSuccess] = useState<boolean | null>(null);

    return (
        <div className="screen screen--daemon">
            <h2>Daemon</h2>
            <Action
                label="Status"
                description="Status of the daemon."
                actionType={ActionType.Status}
                status={daemonIsReachable}
            />
            <Action
                label="Restart daemon"
                description="Restart daemon when functionalities like loading messages or updating settings does not work."
                actionType={ActionType.Reset}
                onClick={async () => {
                    const success = await restartDaemon();
                    setRestartDaemonSuccess(success);
                }}
                success={restartDaemonSuccess}
            />
        </div>
    );
}
