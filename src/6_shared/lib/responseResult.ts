export type Result<T> =
  | { data: T; error: undefined }
  | { data: undefined; error: unknown }
  | { data: undefined; error: undefined };

export const toResponseResult = async <R, A>(
  func: (...args: A[]) => Promise<R>,
  ...args: A[]
): Promise<Result<R>> => {
  try {
    const data = await func(...args);
    return { data, error: undefined };
  } catch (error) {
    return { data: undefined, error };
  }
};
