import React from "react";
import { useState, useEffect } from "react";
import { useClient } from "../../hooks/useClient";
import Action from "../../components/Action/Action";
import { ActionType } from "../../components/Action/ActionType";

export default function Settings()  {
    const { resetTorCircuit } = useClient();
    const [success, setSuccess] = useState<boolean | null>(null);

    return (
        <div className="screen screen--settings">
            <h2>Settings</h2>
            <Action
                label="Reset Tor circuit"
                description="Make Tor client use new circuit to solve unstable connection."
                actionType={ActionType.Delete}
                onClick={async () => {
                    const success = await resetTorCircuit();
                    setSuccess(success);
                }}
                success={success}
            />
        </div>
    );
}
