// This file contains some statistics functions and helpers.

/**
 * Initiates a ring buffer with the given capacity.
 * @param capacity The number of items to hold before returning anything.
 */
export function Ring<T>(capacity: number) {
  const data: (T | undefined)[] = Array(capacity).fill(undefined);
  let wIndex = 0;

  return function next(item: T): T | undefined {
    const tmp = data[wIndex];
    data[wIndex++] = item;
    if (wIndex === capacity) wIndex = 0;
    return tmp;
  };
}

/**
 * Creates a simple-moving-average indicator.
 * @param period The maximum number of recent items to consider in each iteration.
 */
export function MovingAverage(period: number) {
    const ring = Ring<number>(period);
    let sum = 0;
    let size = 0;

    return function next(x: number): number {
        const tmp = ring(x);
        if (tmp !== undefined) sum -= tmp;
        sum += x;
        const n = size < period ? ++size : period;
        return sum / n;
    }
}

/**
 * Compute the z-score (standard distance from average) of an iteration of numbers.
 * @param period The maximum number of recent numbers to consider in each iteration.
 */
export function ZScore(period: number) {
  const ring = Ring<number>(period);
  let sum = 0;
  let sum2 = 0;
  let size = 0;

  return function next(x: number): number {
    const tmp = ring(x);
    if (tmp !== undefined) {
      sum -= tmp;
      sum2 -= tmp * tmp;
    }
    sum += x;
    sum2 += x * x;
    const n = size < period ? ++size : period;
    const avg = sum / n;
    const stddev = ((sum2 - sum * avg) / n) ** 2;
    return (x - avg) / stddev;
  };
}
