"use client";

import React, { createContext, useState, useEffect } from "react";

const ClientIdContext = createContext("client_id");

// Generate a random client ID
function generateClientId() {
  return Math.random().toString(36).substring(2, 15);
}

// Create a provider component
export function ClientIdProvider({ children }) {
  const [clientId, setClientId] = useState("");

  useEffect(() => {
    const id = generateClientId();
    setClientId(id);
  }, []);

  return (
    <ClientIdContext.Provider value={clientId}>
      {children}
    </ClientIdContext.Provider>
  );
}

export default ClientIdContext;
