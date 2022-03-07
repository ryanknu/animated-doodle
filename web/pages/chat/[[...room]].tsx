import type { NextPage } from "next";
import Head from "next/head";
import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";
import { ChatRoom } from "../../components/chat-room";
import { useIdentity } from "../../hooks/identity";
import { useRoomControl } from "../../hooks/room-control";

const Chat: NextPage = () => {
  const [newRoomName, setNewRoomName] = useState("");
  const router = useRouter();
  const { room } = router.query;
  const [userId, _userName, _a, _b, logOut] = useIdentity();
  const [rooms, currentRoom, createRoom, roomWorkingError] = useRoomControl(
    `${room}`
  );

  // Clear new room name textbox when new rooms are added
  useEffect(() => {
    setNewRoomName("");
  }, [rooms]);

  return (
    <div>
      <Head>
        <title>chat-app</title>
        <meta name="description" content="Prank your friends with chat-app" />
      </Head>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "auto 220px 740px auto",
        }}
      >
        <div />
        <div>
          <div>
            <a onClick={logOut}>Log Out</a>
          </div>
          <h2>rooms</h2>
          <ul>
            {rooms.map((r) => {
              return currentRoom && r.id === currentRoom.id ? (
                <li key={r.id}>#{r.properties.name} [selected]</li>
              ) : (
                <li key={r.id}>
                  <Link href={`/chat${r.id.substring(r.id.lastIndexOf("/"))}`}>
                    <a>#{r.properties.name}</a>
                  </Link>
                </li>
              );
            })}
          </ul>
          <h2>create room</h2>
          <div>
            <input
              type="text"
              value={newRoomName}
              onChange={(e) => setNewRoomName(e.currentTarget.value)}
            />
            <button onClick={() => createRoom(newRoomName)}>create room</button>
          </div>
          {roomWorkingError === true ? "loading" : null}
          {roomWorkingError !== false ? roomWorkingError : null}
        </div>
        <div>
          {currentRoom && userId && (
            <ChatRoom room={currentRoom} userId={userId} />
          )}
        </div>
        <div />
      </div>
    </div>
  );
};

export default Chat;
