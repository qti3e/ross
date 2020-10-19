import { Connection } from "./connection";
import { syncClock } from "./clock";
import { rnd32 } from "./utils";

/**
 * An active editor session in the VCS, it exposes methods to manipulate
 * the data on the servers side, and/or fetch the data, while keeping all
 * parties in sync.
 */
export class Session extends EventTarget {
  /**
   * The WebSocket connection which is used to communicate with the server.
   * @readonly
   */
  connection;

  /**
   * The interval descriptor of the task that syncs the clock.
   * @private @readonly
   */
  clockIntervalId;

  /**
   * Construct a new session instance.
   * @param {string} serverURL The ws url of the server which we should connect.
   */
  constructor(serverURL) {
    this.connection = new Connection(serverURL);
    this.handleMessage = this.handleMessage.bind(this);
    this.connection.addEventListener("message", this.handleMessage);
    this.handleOpened = this.handleOpened.bind(this);
    this.connection.addEventListener("opened", this.handleOpened);
    this.handleClosed = this.handleClosed.bind(this);
    this.connection.addEventListener("closed", this.handleClosed);

    // Sync the clock now and every 15 seconds from now.
    this.syncClock();
    this.clockIntervalId = setTimeInterval(() => {
      this.syncClock();
    }, 15e3);
  }

  /**
   * Sync the clock.
   * @private
   */
  async syncClock() {
    if (!this.connection.isOpen()) return;
    if (this.clockSynInProgress === true) return;
    this.clockSynInProgress = true;
    this.clockOffset = await syncClock(this.connection);
    this.clockSynInProgress = false;
  }

  /**
   * Return the current server time in ms. This is the equivalent of `Date.now()` but
   * the clock is synced across server and client.
   */
  now() {
    if (this.clockOffset === undefined)
      throw new Error("Clock is not synced with the server yet.");
    return Date.now() + this.clockOffset;
  }

  /**
   * Create and return a new identifier that is unique across the DB.
   */
  uuid() {
    if (!this.hostID) throw new Error("HostID is not assigned yet.");
    const buffer = new ArrayBuffer(16);
    const view = new DataView(buffer);
    const time = this.now();
    // Get the high bits of the current time.
    view.setFloat64(0, time, true);
    const a = view.getUint32(4);
    // Generate the uuid.
    view.setFloat64(0, time, false);
    view.setUint32(4, this.hostID, false);
    view.setUint32(8, rnd32(), false);
    view.setUint32(12, a, false);
    const p = (n: number) =>
      view
        .getUint32(n * 4, false)
        .toString(16)
        .padStart(8, "0");
    return p(0) + p(1) + p(2) + p(3);
  }

  /**
   * Close the current session.
   */
  close() {
    this.connection.close();
    clearInterval(this.clockIntervalId);
  }

  /**
   * Handle the `opened` event sent from the connection.
   */
  handleOpened() {}

  /**
   * Handle the `closed` event sent from the connection.
   */
  handleClosed() {
    this.hostID = undefined;
  }

  /**
   * Handle messages sent from the server.
   * @param {MessageEvent<Object>} event The new message.
   */
  handleMessage({ data }) {
    if (data._h) {
      // This is just a host id message, so just set the `hostID` and return.
      this.hostID = data._h;
      return;
    }

    // This is a clock sync message that is sent from the server in order to
    // sync the clock, so just ignore it here, this is handled in `clock.js`.
    if (data.clockSync) return;

    // TODO(nima) Handle the actions sent by the server.
    switch (data.action) {
      case "CREATE":
      case "DELETE":
      case "CAS":
    }
  }
}
