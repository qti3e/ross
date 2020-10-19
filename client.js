/**
 * Initialize the client returning a application.
 * 
 * @param {string} serverURL 
 */
function initializeApp(serverURL) {
  const objects = new Map();
  let hostID;

  /**
   * Return the current time.
   */
  function now() {}

  /**
   * Create a new unique identifer that can be used as object key.
   */
  function uuid() {}

  const ws = new WebSocket(serverURL);

  ws.onerror = () => {
    // Handle the error.
  };

  ws.onclose = () => {
    // Handle the close event.
  };

  ws.onopen = () => {};

  ws.onmessage = (msg) => {
    const data = msg.data;
    if (!hostID) {
      hostID = data;
      return;
    }

    switch (data.action) {
      case "CREATE":
        handleCreate(data.instance, data.uuid, data.data);
        break;
      case "DELETE":
      case "CAS":
        // Just a set.
    }
  };

  function handleCreate(instance, uuid, data) {}

  function perform(action) {
    // TODO(nima) Handle the offline case.
    ws.send(action);
  }

  return {
    /**
     * Create a new object in the system.
     * 
     * @param {string} instance Name of the instance that we want to construct.
     * @param {*} defaults The default values of the object.
     */
    create(instance, defaults) {
      const id = uuid();
      perform({
        action: "CREATE",
        uuid: id,
        instance,
        data: defaults
      });
      handleCreate(instance, id, defaults);
    },
    /**
     * Compare and Set, change the value of a field if the current value is
     * the provided value.
     * 
     * @param {Object} object 
     * @param {string} field 
     * @param {any} newValue 
     */
    CAS(object, field, newValue) {},
    /**
     * Delete the given object from the database.
     * @param {Object} object The object to be removed.
     */
    delete(object) {},
    /**
     * Return list of all the objects in the database.
     * @returns {Object[]}
     */
    getObjects() {
      return Array.from(objects.values());
    }
    /**
     * Register a callback to be called once a change occurs.
     * @param {string} eventName 
     * @param {() => void} callback 
     */
    on(eventName, callback) {}
  }
}

const db = initializeApp("ws://xx");

db.on("created", object => {
});

db.on("conflict", conflict => {

});

db.create("shape", {
  x: 50,
  y: 30,
  color: "#fff",
  variant: "rectangle",
  scale: 3
})

db.delete(react);

{
  _instance: "shape",
  _uuid: "xxxxx",
  x: 50,
  y: 30,
  color: "#fff",
  variant: "rectangle",
  scale: 3
}