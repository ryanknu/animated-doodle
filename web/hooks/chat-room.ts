import { Room } from './room-control';
/**
 * This hook provides a clean and (most importantly) synchronous data layer
 * for the front end to use, representing a chat room. This simplifies the
 * design of the front end dramatically and makes it a lot safer to perform
 * state updates.
 */
import { useEffect, useState } from "react"
import { interval } from "rxjs"

interface IMessage {
  date_time: string,
  sender_name: string,
  message: string,
}
interface Message {
  id: string,
  properties: IMessage,
}

type SendMessageHandler = (message: string, senderId: string) => Promise<void>
type ChatRoom = [string, Message[], SendMessageHandler]

export function useChatRoom(props: Room|null): ChatRoom {
  const [timer, setTimer] = useState(0)
  const [messages, setMessages] = useState<Message[]>([])

  useEffect(() => {
    const s = interval(5000).subscribe(ticks => setTimer(ticks))
    return () => s?.unsubscribe()
  }, [])

  useEffect(() => {
    setMessages([])
    setTimer(timer + 1)
  }, [props])

  // Check for messages and set them into state. Since this is asynchronous,
  // take care not to call setState on an unmounted component!!
  useEffect(() => {
    if (props !== null) {
      let mounted = true
      getMessages(props.properties.messages).then(messages => mounted && setMessages(messages.reverse()))
      return () => { mounted = false }
    }
  }, [timer])

  return [
    props?.properties.name || "",
    messages,
    async (message: string, senderId: string) => {
      if (props !== null) {
        await postMessage(props.properties.messages, message, senderId)
        setTimer(timer + 1)
      }
    }
  ]
}

async function getMessages(uri: string): Promise<Message[]> {
  try {
    const res = await fetch(uri)
    return await res.json()
  } catch (_e) {
    return []
  }
}

async function postMessage(uri: string, message: string, senderId: string): Promise<void> {
  const res = await fetch(uri, {
    method: "PUT",
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify({
      message, sender_id: senderId
    })
  })
  const text = await res.json()
}
