import { md5 } from "./md5";

// TODO(qti3e) Remove the use of strings from MD5.

const e = eval;
const global = e("this");

export const enum RandBackend {
  Crypto,
  Hash,
}

/**
 * A random number generator.
 */
export class Rand {
  /**
   * window.crypto or window.msCrypto.
   */
  private crypto?: Crypto;

  /**
   * The numbers that were given to this generator via `feed()`.
   */
  private feeded: number[] = [];

  /**
   * Cache 32 U32.
   */
  private cache?: Uint8Array;

  /**
   * The current offset in `cache`.
   */
  private cursor = 0;

  constructor() {
    this.crypto = getAvailableCrypto();
  }

  /**
   * The random generator has two backend (a.k.a modes) one uses browser's
   * Crypto, the other uses a combination of MD5 and Math.random() and custom
   * feeding (You can feed mouse move and other stuff like that).
   */
  get backend(): RandBackend {
    return this.crypto ? RandBackend.Crypto : RandBackend.Hash;
  }

  feed(...n: number[]): void {
    this.feeded.push(...n);
    while (this.feeded.length > 16) this.feeded.shift();
  }

  private fromFeed(): string {
    if (this.feeded.length === 0) {
      const r = Math.random;
      return r() + "-" + r() + "#" + r() + "#" + r() + "-" + r();
    }
    return this.feeded.pop() + this.feeded.join("-");
  }

  /**
   * Returns a random U8.
   */
  rnd8(): number {
    if (this.cache && this.cursor < this.cache.byteLength) {
      return this.cache[this.cursor++];
    }

    this.cursor = 0;

    if (this.crypto) {
      if (!this.cache) this.cache = new Uint8Array(128);
      this.crypto.getRandomValues(this.cache);
      return this.rnd8();
    }

    let newSet: string;

    if (!this.cache) {
      this.cache = new Uint8Array(128);
      const a = md5(this.fromFeed());
      const b = md5(a + this.fromFeed());
      const c = md5(a + b + Math.random());
      newSet = c + md5(a + b + c);
    } else {
      const a = md5(this.cache.join("-"));
      const b = md5(a + this.fromFeed());
      newSet = b + md5(b + a);
    }

    for (let i = 0; i < 32; i += 1) {
      const n = parseInt(newSet.substr(i * 2, 2), 16);
      this.cache[i] = n;
    }

    return this.rnd8();
  }

  /**
   * Returns a random U32 number.
   */
  rnd32(): number {
    let r = 0;
    r += this.rnd8();
    r += this.rnd8() << 8;
    r += this.rnd8() << 16;
    r += this.rnd8() << 24;
    return r;
  }
}

function getAvailableCrypto(): Crypto | undefined {
  const buf = new Uint8Array(1);
  try {
    if ("crypto" in global) {
      global.crypto.getRandomValues(buf);
      return global.crypto;
    } else if ("msCrypto" in global) {
      global.msCrypto.getRandomValues(buf);
      return global.msCrypto;
    }
  } catch (e) {}
  return undefined;
}

export const rnd = new Rand();
