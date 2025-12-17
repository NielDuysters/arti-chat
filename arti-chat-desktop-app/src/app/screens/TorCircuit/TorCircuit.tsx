import React from "react";
import { useState, useEffect } from "react";
import { useClient } from "../../hooks/useClient";
import Action from "../../components/Action/Action";
import { ActionType } from "../../components/Action/ActionType";

export default function TorCircuit()  {
    const { resetTorCircuit } = useClient();
    const [resetTorCircuitSuccess, setResetTorCircuitSuccess] = useState<boolean | null>(null);

    return (
        <div className="screen screen--tor-circuit">
            <h2>Tor circuit</h2>
            <Action
                label="Reset Tor circuit"
                description="Make Tor client use new circuit to solve unstable connection."
                actionType={ActionType.Reset}
                onClick={async () => {
                    const success = await resetTorCircuit();
                    setResetTorCircuitSuccess(success);
                }}
                success={resetTorCircuitSuccess}
            />
        </div>
    );
}
