import type { NextPage } from "next";
import Head from "next/head";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";
import { useApiHealthChecker } from "../hooks/api-health-checker";
import { useIdentity } from "../hooks/identity";
import styles from "../styles/Home.module.css";

const Home: NextPage = () => {
  const [name, setName] = useState("");
  const [userId, _userName, signUp, signIn, _signOut, workingError] =
    useIdentity();
  const apiIsHealthy = useApiHealthChecker();
  const router = useRouter();

  useEffect(() => {
    if (userId !== null) {
      router.replace("/chat");
    }
  }, [userId]);

  return (
    <div className={styles.container}>
      <Head>
        <title>chat-app</title>
        <meta name="description" content="Prank your friends with chat-app" />
      </Head>

      <main className={styles.main}>
        <h1 className={styles.title}>Chat App</h1>

        <p className={styles.description}>What's your name?</p>

        <div>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.currentTarget.value)}
          />
        </div>

        <button onClick={() => signUp(name)}>Sign up</button>
        <button onClick={() => signIn(name)}>Sign in</button>

        {workingError === true ? "loading" : null}
        {workingError !== false ? workingError : null}
      </main>

      <footer className={styles.footer}>
        API Status: {apiIsHealthy ? "OK" : "Unavailable??"}
      </footer>
    </div>
  );
};

export default Home;
