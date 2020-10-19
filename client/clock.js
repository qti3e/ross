import { Connection } from "./connection";
import { createDeferred, mid, mean, stddev } from "./utils";

const CLOCK_OFFSET_REQ_PER_SYNC = 9;

/**
 * Try to compute the clock offset of client and server over a
 * connection.
 * @param {Connection} connection
 * @returns {Promise<number>}
 */
export async function syncClock(connection) {
  const wait = createDeferred();
  const offsets = [];

  function onMessage({ data }) {
    if (!data._t) return;
    const local = data._t;
    const server = data._s;
    const latency = Date.now() - local;
    const offset = server - (local + latency / 2);
    offsets.push(offset);

    if (offsets.length === CLOCK_OFFSET_REQ_PER_SYNC) wait.resolve();
  }

  function onClose() {
    wait.reject();
  }

  connection.addEventListener("message", onMessage);
  connection.addEventListener("closed", onMessage);

  try {
    for (let i = 0; i < CLOCK_OFFSET_REQ_PER_SYNC; ++i)
      await connection.send({ _t: Date.now() });
  } finally {
    connection.removeEventListener("message", onMessage);
    connection.removeEventListener("closed", onMessage);
  }

  await wait;

  // Sort the time offsets to compute the `mid`.
  const max = mid(offsets) + stddev(offsets);
  const finalOffset = mean(offsets.filter(n => n < max));
  if (Number.isNaN(finalOffset) || !Number.isFinite(finalOffset))
    throw new Error("Clock sync failed");
  return finalOffset;
}
