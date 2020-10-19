import { createDeferred } from "./utils";

/**
 * A WS client that handles connection errors properly.
 */
export class Connection extends EventTarget {
  /**
   * @type string
   * @readonly
   */
  serverURL;

  /**
   * The current active websocket instance.
   * @type WebSocket
   * @private
   */
  connection;

  /**
   * Wether the current connection was ever successfully established or not.
   */
  everOpened = false;

  /**
   * The ready-state of this connection.
   * @type "PENDING" | "OPEN" | "CLOSED"
   */
  readyState = "PENDING";

  /**
   * An incremental counter used as the replyId field of each message, because
   * WebSocket has no way of telling us wether the server has indeed received
   * a message or not, we attach a `_` field to each outgoing message, which
   * is our `replyId`, once the server receives a message and successfully
   * processes our request/command we get back a `reply` message from the
   * server this message indicates that our message has successfully been
   * processed and in that instant, we resolve the deferred promise returned
   * by the `send` called that initiated the command.
   * @type number
   * @private
   */
  replyIdCounter = 0;

  /**
   * Map each replyId to a deferred promise that was returned by the
   * send function.
   * @private
   */
  deferredReplyMap = new Map();

  constructor(serverURL) {
    super();
    this.serverURL = serverURL;
    this.handleOnOpen = this.handleOnOpen.bind(this);
    this.handleOnError = this.handleOnError.bind(this);
    this.handleOnClose = this.handleOnClose.bind(this);
    this.handleOnMessage = this.handleOnMessage.bind(this);
    tryOpen();
  }

  /**
   * Try starting a new connection.
   */
  tryOpen() {
    if (this.isOpen()) return;

    this.readyState = "PENDING";
    this.connection = new WebSocket(this.serverURL);
    this.connection.onopen = this.handleOnOpen;
    this.connection.onerror = this.handleOnError;
    this.connection.onclose = this.handleOnClose;
    this.connection.onmessage = this.handleOnMessage;
  }

  /**
   * Close the current connection.
   */
  close() {
    if (!this.connection) return;

    this.everOpened = false;
    this.readyState = "CLOSED";
    // Disable its event listeners.
    this.connection.onopen = this.connection.onerror = this.connection.onclose = this.connection.onmessage = undefined;
    this.connection = undefined;
    this.connection.close();
    this.rejectAllPendingSends();

    if (this.everOpened) {
      this.dispatchEvent(new CustomEvent("closed"));
    }
  }

  /**
   * Wether this connection is open, and we can send data to the server.
   */
  isOpen() {
    return (
      this.readyState === "OPEN" &&
      this.connection &&
      this.connection.readyState === this.connection.OPEN
    );
  }

  /**
   * Send the given data to the server, returns a promise that will be resolved once
   * the message is delivered to the server.
   * @param {Object} data
   * @return
   */
  send(data) {
    if (!this.isOpen())
      throw new Error("Cannot invoke send on an inactive connection.");
    const replyId = this.replyIdCounter++;
    // Serialize and send the message.
    const msg = JSON.stringify({ ...data, _r: replyId });
    this.connection.send(msg);
    // Create and store the associated promise and return it.
    const promise = createDeferred();
    this.deferredReplyMap.set(replyId, promise);
    return promise;
  }

  /**
   * This is an internal method to reject all of the pending promises that
   * were returned by `send`, this method is called once the connection
   * status changes upon an error or any other reason.
   * @private
   */
  rejectAllPendingSends() {
    for (const deferred of this.deferredReplyMap.values()) deferred.reject();
    this.deferredReplyMap.clear();
  }

  /**
   * Handle the `open` event of the underlying WebSocket connection.
   * @private
   */
  handleOnOpen() {
    this.everOpened = true;
    this.dispatchEvent(new CustomEvent("opened"));
  }

  /**
   * Handle the `error` event of the underlying WebSocket connection.
   * @private
   */
  handleOnError() {
    this.close();
  }

  /**
   * Handle the `close` event of the underlying WebSocket connection.
   * @private
   */
  handleOnClose() {
    this.close();
  }

  /**
   * Handle incoming messages.
   * @param {WebSocketEventMap["message"]} event The message event.
   * @private
   */
  handleOnMessage(event) {
    try {
      const data = JSON.parse(event.data);

      if (data._r) {
        const deferred = this.deferredReplyMap.get(data._r);
        if (!deferred) throw new Error("Unexpected error occurred.");
        deferred.resolve();
        this.deferredReplyMap.delete(data._r);
        // This was just a reply message, and is considered an internal message, so
        // we won't proceed it any further.
        return;
      }

      this.dispatchEvent(new MessageEvent("message", { data }));
    } catch (e) {
      this.dispatchEvent(
        new CustomEvent("parse-error", {
          detail: event.data
        })
      );
    }
  }
}
