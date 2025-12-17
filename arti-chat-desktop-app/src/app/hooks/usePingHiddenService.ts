import { useEffect, useRef, useState } from "react";
import { useClient } from "./useClient";

const PING_INTERVAL_MS = 60000;

export function useHiddenServicePing() {
    const { pingHiddenService } = useClient();

    const [isReachable, setIsReachable] = useState<boolean | null>(null);
    const intervalRef = useRef<number | null>(null);

    useEffect(() => {
        let cancelled = false;

        const ping = async () => {
            try {
                const result = await pingHiddenService();
                if (!cancelled) {
                    setIsReachable(result);
                }
            } catch {
                if (!cancelled) {
                    setIsReachable(false);
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
    }, [pingHiddenService]);

    return {
        isReachable,
    };
}

