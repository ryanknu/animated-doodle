import React, { useEffect, useRef, useState } from "react";
import { useChatRoom } from "../hooks/chat-room";
import { Room } from "../hooks/room-control";

interface ChatRoomProps {
  room: Room;
  userId: string;
}

export function ChatRoom(props: ChatRoomProps) {
  const [newMessage, setNewMessage] = useState("");
  const [roomName, messages, postMessage] = useChatRoom(props.room);
  const scrollbackContainer = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollbackContainer.current) {
      scrollbackContainer.current.scrollTo(
        0,
        scrollbackContainer.current.scrollHeight
      );
    }
  }, [messages]);

  return (
    <div>
      <div>
        <div
          style={{ overflowY: "scroll", maxHeight: "90vh" }}
          ref={scrollbackContainer}
        >
          {messages.map((m) => (
            <p key={m.id}>
              {m.properties.sender_name}:{m.properties.message}(
              {m.properties.date_time})
            </p>
          ))}
        </div>
        <div>You're talking in #{roomName}.</div>
        <div>
          <input
            type="text"
            value={newMessage}
            onChange={(e) => setNewMessage(e.currentTarget.value)}
          />
          <button onClick={() => postMessage(newMessage, props.userId)}>
            Send
          </button>
        </div>
      </div>
    </div>
  );
}
