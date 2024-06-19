"use client";

import { useContext, useEffect } from "react";
import { useSearchParams } from "next/navigation";
import ClientIdContext from "../ClientIdContext";

const MyProfile = () => {
  const searchParams = useSearchParams();
  const state = searchParams.get("state");
  const code = searchParams.get("code");
  const authApiBaseUrl = process.env.NEXT_PUBLIC_AUTH_API_BASE_URL;
  const clientId = useContext(ClientIdContext);

  useEffect(() => {
    if (state && code) {
      fetch(authApiBaseUrl + "/auth", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "X-Client-Id": clientId,
        },
        body: JSON.stringify({ state, code, id: clientId }),
      })
        .then((response) => response.json())
        .then((data) => console.log(data))
        .catch((error) => console.error("Error:", error));
    }
  }, [state, code]);

  return (
    <div>
      <section className="w-full flex-center flex-col">
        <h1 className="head_text text-center orange_gradient">
          A super amazing and personalized page
        </h1>
      </section>
    </div>
  );
};

export default MyProfile;
