import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface Contact {
  onion: string;
  nickname: string;
}

export function useContacts() {
  const [contacts, setContacts] = useState<Contact[]>([]);

  useEffect(() => {
    loadContacts();
  }, []);

  async function loadContacts() {
    const list = await invoke<Contact[]>("load_contacts");
    setContacts(list);
  }

  return {
    contacts,
  };
}

