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
    super();
    this.connection = new Connection(serverURL);
    this.handleMessage = this.handleMessage.bind(this);
    this.connection.addEventListener("message", this.handleMessage);
    this.handleOpened = this.handleOpened.bind(this);
    this.connection.addEventListener("opened", this.handleOpened);
    this.handleClosed = this.handleClosed.bind(this);
    this.connection.addEventListener("closed", this.handleClosed);
  }

  /**
   * Sync the clock.
   * @private
   */
  async syncClock() {
    if (!this.connection.isOpen()) return;
    if (this.clockSynInProgress === true) return;
    try {
      this.clockSynInProgress = true;
      const offset = await syncClock(this.connection);
      this.clockOffset = offset;
    } catch (e) {
      // Ignore clock sync failures.
    } finally {
      this.clockSynInProgress = false;
    }
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
  }

  /**
   * Handle the `opened` event sent from the connection.
   */
  handleOpened() {
    // Sync the clock now and every 15 seconds from now.
    this.syncClock();
    this.clockIntervalId = setTimeInterval(() => {
      this.syncClock();
    }, 15e3);
  }

  /**
   * Handle the `closed` event sent from the connection.
   */
  handleClosed() {
    this.hostID = undefined;
    clearInterval(this.clockIntervalId);
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

    // TODO(nima) Handle the actions sent by the server, remember that the
    // server is our source of truth and client always does what the server
    // says.
    switch (data.action) {
      case "CREATE":
      case "DELETE":
      case "CAS":
        throw new Error("unimplemented.");
      // This action is sent by the server at the beginning of the sync.
      // It contains all of the objects in the database, all of the data
      // (if any) must be deleted and be replaced by the data provided
      // in this action.
      case "LOAD":
        throw new Error("unimplemented.");
    }
  }

  /**
   * Perform the given action on the server side, this function should not
   * impact the current instance in anyways.
   * @param {Object} action
   * @private
   */
  async perform(action) {
    if (!this.connection.isOpen()) {
      // TODO(qti3e,nima) Currently this function assumes that the connection
      // is always open, but it might actually be closed, in that case we need
      // to store this actions in some sort of stack, and track our changes
      // using some sort of ds, so that once the connection is established again
      // we can detect conflicts in the sync process in semi-linear time.
      throw new Error("unimplemented.");
    }
    this.connection.send(action);
  }

  /**
   * Create an object with the information provided in this instance,
   * this is a client side function and only affects the client side.
   * @private
   */
  doCreate(instance, uuid, data) {
    const object = Object.freeze({
      _instance: instance,
      _uuid: uuid,
      ...data
    });

    // TODO(nima) This object needs to be stored somewhere.

    this.dispatchEvent(
      new CustomEvent("object-created", {
        detail: object
      })
    );
  }

  /**
   * Create a new object in the current session.
   *
   * @param {string} instance Name of the object instance.
   * @param {Object} data The values of the object.
   */
  create(instance, data) {
    const uuid = this.uuid();
    this.doCreate(instance, uuid, data);
    return this.perform({
      action: "CREATE",
      instance,
      uuid,
      data
    });
  }

  /**
   * Delete an object from this session on the client side, does not affect the
   * server.
   * @param {string} objUUID The uuid of the object that is subject to this action.
   * @private
   */
  doDelete(objUUID) {
    // TODO(nima) Delete the object and dispatch an event named "object-deleted".
    throw new Error("unimplemented.");
  }

  /**
   * Delete
   * @param {Object} object An object that was previously returned by this session.
   */
  delete(object) {
    this.doDelete(object._uuid);
    // TODO(nima) Perform the action on the server side, use `this.perform`
    // just like in `create()`
    throw new Error("unimplemented.");
  }

  /**
   * Set the value of the given field in the given object, this function
   * will perform a CAS action on the server side, returns the newly updated
   * object and dispatch an `object-mutated` event.
   * @param {Object} object
   * @param {string} field
   * @param {any} nextValue
   */
  set(object, field, nextValue) {
    throw new Error("unimplemented.");
  }

  /**
   * Return all of the objects in this session.
   */
  getData() {
    // TODO(nima)
  }
}
