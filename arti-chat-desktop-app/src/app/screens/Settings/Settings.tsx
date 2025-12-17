import React from "react";
import { useState, useEffect } from "react";
import { useClient } from "../../hooks/useClient";
import { useContacts } from "../../hooks/useContacts";
import Action from "../../components/Action/Action";
import { ActionType } from "../../components/Action/ActionType";

export default function Settings({contacts, setContacts})  {
    const { getConfigValue, setConfigValue } = useClient();
    const { deleteAllContacts } = useContacts({contacts: contacts, setContacts: setContacts});
    const [deleteAllContactsSuccess, setDeleteAllContactsSuccess] = useState<boolean | null>(null);
    const [enableNotifications, setEnableNotifications] = useState<boolean>(false);

    useEffect(() => {
        const loadConfig = async () => {
            const enableNotificationsValue = await getConfigValue("enable_notifications");
            setEnableNotifications(enableNotificationsValue === "true")
        };

        loadConfig();
    }, []);

    return (
        <div className="screen screen--settings">
            <h2>Settings</h2>
            
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

            <Action
                label="Enable notifications"
                description="Show desktop notifications for new messages."
                actionType={ActionType.Toggle}
                checked={enableNotifications}
                onClick={async (checked: boolean) => {
                    setEnableNotifications(checked);
                    await setConfigValue("enable_notifications", checked.toString());
                }}
            />
        </div>
    );
}
