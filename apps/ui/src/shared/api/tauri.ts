type SpectaRequestId = string & { readonly __brand: 'RequestId' };

type DropRequestId<T extends unknown[]> = T extends [
  ...infer Rest,
  SpectaRequestId,
]
  ? Rest
  : never;

export const withIdempotency = <
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  Fn extends (...args: any[]) => Promise<unknown>,
>(
  fn: Fn &
    (Parameters<Fn> extends [...unknown[], SpectaRequestId]
      ? unknown
      : [
          'TypeError: The command is missing the RequestId parameter at the end',
        ]),
): ((...args: DropRequestId<Parameters<Fn>>) => ReturnType<Fn>) => {
  const originalFn = fn as unknown as (
    ...args: [...DropRequestId<Parameters<Fn>>, SpectaRequestId]
  ) => ReturnType<Fn>;

  return (...args: DropRequestId<Parameters<Fn>>): ReturnType<Fn> => {
    const requestId = crypto.randomUUID() as SpectaRequestId;
    return originalFn(...args, requestId);
  };
};
