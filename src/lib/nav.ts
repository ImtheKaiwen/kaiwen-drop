/** Klavye ile liste gezintisi (saf fonksiyon). `cur` = -1 hicbiri secili degil. */
export function nextIndex(cur: number, count: number, key: string): number {
  if (count <= 0) return -1;
  switch (key) {
    case "ArrowDown":
      return cur < 0 ? 0 : (cur + 1) % count;
    case "ArrowUp":
      return cur < 0 ? count - 1 : (cur - 1 + count) % count;
    case "Home":
      return 0;
    case "End":
      return count - 1;
    default:
      return cur;
  }
}
