import { generateUUID } from "./uuid/uuid";
import { Hash16 } from "./common";
import { Socket } from "socket.io-client";

export class Session {
  constructor(private readonly io: Socket) {
    this.now = this.now.bind(this);
    this.uuid = this.uuid.bind(this);
  }

  now(): number {
    // TODO(qti3e)
    return Date.now();
  }

  uuid(): Hash16 {
    // TODO(qti3e)
    return generateUUID(this.now());
  }
}
