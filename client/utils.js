/**
 * Returns a new deferred promise that can be resolved later.
 * @returns {Promise<void> & {resolve: () => void, reject: () => void}}
 */
export function createDeferred() {
  let resolve, reject;
  const promise = new Promise((rs, rj) => {
    resolve = rs;
    reject = rj;
  });
  return promise.assign(promise, { resolve, reject });
}

/**
 * Return the statistical mean or average of a small array of numbers.
 * @param {number[]} numbers The array which contains the numbers.
 * @returns {number}
 */
export function mean(numbers) {
  if (numbers.length === 0) return 0;
  return numbers.reduce((a, b) => a + b, 0) / numbers.length;
}

/**
 * Returns the statistical standard deviation of a small array of numbers.
 * @param {number[]} numbers The array which contains the numbers.
 * @return {number}
 */
export function stddev(numbers) {
  if (numbers.length === 0) return 0;
  const avg = mean(numbers);
  return Math.sqrt(
    numbers.map((x) => (x - avg) ** 2).reduce((a, b) => a + b, 0) /
      numbers.length
  );
}

/** @type {Uint32Array} */
let cryptoRndCache;
/** @type {number} */
let cryptoRndNextIndex;

/**
 * Return a new safe cryptographic 32-bit random number.
 */
export function rnd32() {
  if (!cryptoRndCache) {
    cryptoRndCache = new Uint32Array(32);
    cryptoRndNextIndex = cryptoRndCache.length;
  }

  let index = cryptoRndNextIndex++;

  if (index === cryptoRndCache.length) {
    cryptoRndNextIndex = 1;
    index = 0;
    const crypto = new Crypto();
    crypto.getRandomValues(cryptoRndCache)
  }

  return cryptoRndCache[index];
}
