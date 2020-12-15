import { io } from 'socket.io-client';
import { Hash16 } from './common';
import { Session } from './session';

export class Repository {
  readonly baseURL: string;

  constructor(
    serverUrl: string,
    repository: Hash16,
    private readonly token: string
  ) {
    const server = serverUrl + (serverUrl.endsWith("/") ? "" : "/");
    this.baseURL = server + repository + "/";
  }

  async openSession(branch: Hash16): Promise<Session> {
    return new Session(
      io(this.baseURL + "ws", {
        query: {
          token: this.token,
          branch,
        },
      })
    );
  }
}
