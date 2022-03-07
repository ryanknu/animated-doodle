/**
 * TODO
 */

import { useEffect, useMemo, useState } from "react";

const API_URI = process.env.NEXT_PUBLIC_API_URI;
const ROOMS_API_URI = `${API_URI}/rooms`;

interface IRoom {
  name: string;
  messages: string;
}
export interface Room {
  id: string;
  properties: IRoom;
}

type HookResponse = [
  Room[],
  Room | null,
  (_: string) => Promise<void>,
  string | boolean
];

export function useRoomControl(roomId: string): HookResponse {
  const [rooms, setRooms] = useState<Room[]>([]);
  const [workingError, setWorkingError] = useState<string | boolean>(false);

  useEffect(() => {
    let mounted = true;
    listRooms().then((rooms) => mounted && setRooms(rooms));
    return () => {
      mounted = false;
    };
  }, []);

  const currentRoom = useMemo(
    () =>
      rooms.find((r) => r.id.substring(r.id.lastIndexOf("/") + 1) === roomId) ||
      null,
    [roomId, rooms]
  );

  async function listRooms(): Promise<Room[]> {
    let rooms: Room[] = [];
    try {
      const response = await fetch(ROOMS_API_URI);
      rooms = (await response.json()) as Room[];
    } catch (_e) {
      setWorkingError("Failed to fetch rooms");
    }
    return rooms;
  }

  async function createRoom(name: string) {
    if (rooms.findIndex((x) => x.properties.name === name) !== -1) {
      setWorkingError("Room already exists");
      return;
    }

    try {
      const response = await fetch(ROOMS_API_URI, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name }),
      });
      await response.json();
      setWorkingError(false);
      setRooms(await listRooms());
    } catch (_e) {
      setWorkingError("Error creating room.");
    }
  }

  return [rooms, currentRoom, createRoom, workingError];
}
