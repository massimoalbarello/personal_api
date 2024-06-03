"use client";

import { useEffect, useState } from "react";

export default function PapiLogin() {
  const [authUrl, setAuthUrl] = useState("");

  useEffect(() => {
    fetch("http://127.0.0.1:8080/auth", {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
        "X-Client-Id": "123",
      },
    })
      .then((response) => response.json())
      .then((data) => {
        console.log(data["url"]);
        setAuthUrl(data["url"]);
      })
      .catch((error) => console.error("Error:", error));
  }, []);

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
