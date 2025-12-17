import { useEffect, useRef, useState } from "react";
import { useClient } from "./useClient";

const PING_INTERVAL_MS = 10000;

export function useDaemonPing() {
    const { pingDaemon } = useClient();

    const [daemonIsReachable, setDaemonIsReachable] = useState<boolean | null>(null);
    const intervalRef = useRef<number | null>(null);

    useEffect(() => {
        let cancelled = false;

        const ping = async () => {
            try {
                const result = await pingDaemon();
                if (!cancelled) {
                    setDaemonIsReachable(result);
                }
            } catch {
                if (!cancelled) {
                    setDaemonIsReachable(false);
                }
            }
        };

        // Initial ping.
        ping();

        // Periodic ping.
        intervalRef.current = window.setInterval(ping, PING_INTERVAL_MS);

        return () => {
            cancelled = true;
            if (intervalRef.current !== null) {
                clearInterval(intervalRef.current);
            }
        };
    }, [pingDaemon]);

    return {
        daemonIsReachable,
        setDaemonIsReachable,
    };
}

