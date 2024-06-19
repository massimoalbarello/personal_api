"use client";

import { useEffect, useState, useContext } from "react";
import ClientIdContext from "../ClientIdContext";

export default function PapiLogin() {
  const [authUrl, setAuthUrl] = useState("");
  const authApiBaseUrl = process.env.NEXT_PUBLIC_AUTH_API_BASE_URL;
  const clientId = useContext(ClientIdContext);

  useEffect(() => {
    if (clientId) {
      console.log("Auth API base URL:", authApiBaseUrl);
      console.log("Client Id:", clientId);
      fetch(authApiBaseUrl + "/auth", {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
          "X-Client-Id": clientId,
        },
      })
        .then((response) => response.json())
        .then((data) => {
          console.log("Google OAuth2 URL:", data["url"]);
          setAuthUrl(data["url"]);
        })
        .catch((error) => console.error("Error:", error));
    }
  }, [clientId]); // the first time the request is sent, clientId might not have been set yet. Make sure to sentd it again once the clientId is set

  const handleRedirect = () => {
    // Redirect to Google OAuth2 URL
    window.location.href = authUrl;
  };

  return (
    <div>
      <button onClick={handleRedirect}>Personalize this page</button>
    </div>
  );
}
