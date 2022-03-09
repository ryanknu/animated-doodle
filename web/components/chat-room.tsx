import React, { useEffect, useRef, useState } from "react";
import { Message, useChatRoom } from "../hooks/chat-room";
import { Room } from "../hooks/room-control";

interface ChatRoomProps {
  room: Room;
  userId: string;
  userName: string;
}

export function ChatRoom(props: ChatRoomProps) {
  const [newMessage, setNewMessage] = useState("");
  const [roomName, messages, postMessage] = useChatRoom(props.room);
  const scrollbackContainer = useRef<HTMLDivElement>(null);
  const [displayedMessages, setDisplayedMessages] = useState<Message[]>([])

  // scroll to bottom of chat window on message received
  useEffect(() => {
    if (scrollbackContainer.current) {
      scrollbackContainer.current.scrollTo(
        0,
        scrollbackContainer.current.scrollHeight
      );
    }
  }, [displayedMessages]);

  // merge incoming messages into in-memory messages
  useEffect(() => {
    let needsSort = false
    for (const message of messages) {
      const found = displayedMessages.find(x => x.id === message.id)
      if (found === undefined) {
        displayedMessages.push(message)
        needsSort = true
      }
    }
    // sort them to avoid any race conditions
    if (needsSort) {
      displayedMessages.sort((a, b) => a.properties.date_time.localeCompare(b.properties.date_time))
      // remove temporary messages
      const copy = displayedMessages.filter(a => a.id !== "unknown")
      setDisplayedMessages(copy)
    }
  }, [messages])

  // empty out message cache when the room changes
  useEffect(() => {
    setDisplayedMessages([])
  }, [props.room])

  function send() {
    postMessage(newMessage, props.userId)
    // update local message copy
    setDisplayedMessages(displayedMessages.concat([{
      id: "unknown",
      properties: {
        sender_name: props.userName,
        date_time: (new Date()).toISOString(),
        message: newMessage,
      }
    }]))
    setNewMessage("")
  }

  return (
    <div>
      <div>
        <div
          style={{ overflowY: "scroll", maxHeight: "90vh" }}
          ref={scrollbackContainer}
        >
          {displayedMessages.map((m) => (
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
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                send()
              }
            }}
          />
          <button onClick={send}>
            Send
          </button>
        </div>
      </div>
    </div>
  );
}
