/**
 * Reads `"..."` that opens at `start`, where `""` is a literal quote. Undefined when the quote is
 * never closed.
 */
export const readQuoted = (value: string, start: number) => {
  let result = '';

  for (let i = start + 1; i < value.length; i++) {
    if (value[i] !== '"') {
      result += value[i];
    } else if (value[i + 1] === '"') {
      result += '"';
      i++;
    } else {
      return { value: result, end: i + 1 };
    }
  }

  return undefined;
};

export const quote = (value: string) => `"${value.replaceAll('"', '""')}"`;
