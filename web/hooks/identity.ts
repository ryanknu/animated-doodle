/**
 * This hook exposes an rxjs observer that can be subscribed to that
 * continually checks the API server's status check endpoint.
 * In a production app I would have this thing adjust it's interval
 * timer and increase the frequency when interesting things are
 * happening, and decrease it when nothing is happening.
 *
 * To do this I'd add another state value for frequency, and then
 * increase the interval timer to be 1 second, and do a timestamp
 * check during each fire to allow me to more precisely control the delay.
 */

import { useEffect, useState } from "react";

const API_URI = process.env.NEXT_PUBLIC_API_URI;
const SIGN_UP_URI = `${API_URI}/sign-up`;
const SIGN_IN_URI = `${API_URI}/sign-in`;

type Identity = [string | null, string | null];
type HookResponse = [
  string | null,
  string | null,
  (_: string) => Promise<void>,
  (_: string) => Promise<void>,
  () => void,
  string | boolean
];

const defaultIdentity = (): Identity => (typeof window !== "undefined") ? [
  localStorage.getItem("com.ryanknu.chat-app__id"),
  localStorage.getItem("com.ryanknu.chat-app__name"),
] : [null, null];

export function useIdentity(): HookResponse {
  const [identity, setIdentity] = useState<Identity>(defaultIdentity());
  const [apiRequest, setApiRequest] = useState<Promise<Response> | null>(null);
  const [workingError, setWorkingError] = useState<string | boolean>(false);

  async function sign(direction: "up" | "in", name: string) {
    const uri = direction === "up" ? SIGN_UP_URI : SIGN_IN_URI;
    setWorkingError(true);
    setApiRequest(
      fetch(`${uri}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name }),
      })
    );
  }

  function signOut() {
    localStorage.clear();
    setWorkingError(false);
    setIdentity([null, null]);
  }

  const signUp = (name: string) => sign("up", name);
  const signIn = (name: string) => sign("in", name);

  useEffect(() => {
    let mounted = true;
    apiRequest
      ?.then((response) => response.text())
      .then((text) => {
        if (text === "Result DNE or is ambiguous") {
          setWorkingError("User already exists. Sign in instead.")
          return
        }
        if (text === "Name is empty") {
          setWorkingError("Please fill in your name.")
          return
        }
        let data = JSON.parse(text);
        localStorage.setItem("com.ryanknu.chat-app__id", data.id);
        localStorage.setItem(
          "com.ryanknu.chat-app__name",
          data.properties.name
        );
        setIdentity([data.id, data.properties.name]);
      })
      .catch(() => setWorkingError("An error occurred"));
    return () => {
      mounted = false;
    };
  }, [apiRequest]);

  return [identity[0], identity[1], signUp, signIn, signOut, workingError];
}
