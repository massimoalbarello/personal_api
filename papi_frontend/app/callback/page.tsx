"use client";

import { useEffect } from "react";
import { useSearchParams } from "next/navigation";

const MyProfile = () => {
  const searchParams = useSearchParams();
  const state = searchParams.get("state");
  const code = searchParams.get("code");

  useEffect(() => {
    if (state && code) {
      fetch("http://127.0.0.1:8080/auth", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "X-Client-Id": "123",
        },
        body: JSON.stringify({ state, code, id: "123" }),
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
