import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface Message {
    body: string;
    timestamp: number;
}

export function useChat(activeContact) {
  const [messages, setMessages] = useState<Message[]>([]);


  const loadChat = useCallback(async () => {
    if (!activeContact) return;

    const msgs = await invoke<Message[]>("load_chat", {
      onionId: activeContact.onion_id,
    });

    setMessages(msgs);
  }, [activeContact]);
  
  useEffect(() => {
    loadChat();
  }, []);

  return {
    messages,
  };
}

