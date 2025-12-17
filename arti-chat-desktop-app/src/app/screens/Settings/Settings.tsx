import React from "react";
import { useState, useEffect } from "react";
import { useClient } from "../../hooks/useClient";
import { useContacts } from "../../hooks/useContacts";
import Action from "../../components/Action/Action";
import { ActionType } from "../../components/Action/ActionType";

export default function Settings({contacts, setContacts})  {
    const { resetTorCircuit } = useClient();
    const { deleteAllContacts } = useContacts({contacts: contacts, setContacts: setContacts});
    const [resetTorCircuitSuccess, setResetTorCircuitSuccess] = useState<boolean | null>(null);
    const [deleteAllContactsSuccess, setDeleteAllContactsSuccess] = useState<boolean | null>(null);

    return (
        <div className="screen screen--settings">
            <h2>Settings</h2>
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
            
            <Action
                label="Delete all contacts"
                description="Delete all contacts and messages."
                actionType={ActionType.Delete}
                onClick={async () => {
                    const success = await deleteAllContacts();
                    setDeleteAllContactsSuccess(success);
                }}
                success={deleteAllContactsSuccess}
            />
        </div>
    );
}
