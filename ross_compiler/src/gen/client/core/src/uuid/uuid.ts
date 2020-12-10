import { rnd } from "./rand";
import { Hash16 } from "../common";

/**
 * Return a random unique identifier.
 * @param time The current time in ms.
 * @param hostID A unique U32 number for this machine.
 */
export function generateUUID(time: number, hostID?: number): Hash16 {
  const buffer = new ArrayBuffer(16);
  const view = new DataView(buffer);

  view.setFloat64(0, time, true);
  const a = view.getUint32(4);

  view.setFloat64(0, time, false);
  if (hostID === undefined) {
    view.setUint32(4, rnd.rnd32(), false);
  } else {
    view.setUint32(4, hostID, false);
  }
  view.setUint32(8, rnd.rnd32(), false);
  view.setUint32(12, a, false);

  const p = (n: number) =>
    view
      .getUint32(n * 4, false)
      .toString(16)
      .padStart(8, "0");

  return p(0) + p(1) + p(2) + p(3);
}
