"use client";

import React, { createContext, useState, useEffect } from "react";

const ClientIdContext = createContext("clientId");

// Generate a random client ID
const generateClientId = () => {
  return Math.random().toString(36).substring(2, 15);
};

// Create a provider component
export const ClientIdProvider = ({ children }) => {
  const [clientId, setClientId] = useState(null);

  useEffect(() => {
    // Generate the client ID only once
    const storedClientId = localStorage.getItem("clientId");
    if (storedClientId) {
      setClientId(storedClientId);
    } else {
      const id = generateClientId();
      localStorage.setItem("clientId", id);
      setClientId(id);
    }
  }, [clientId]);

  return (
    <ClientIdContext.Provider value={clientId}>
      {children}
    </ClientIdContext.Provider>
  );
};

export default ClientIdContext;
