import { getOwner, onCleanup } from 'solid-js';
import { isServer } from 'solid-js/web';

export type ScheduleCallback = <Args extends unknown[]>(
  callback: (...args: Args) => void,
  wait?: number,
) => Scheduled<Args>;
export interface Scheduled<Args extends unknown[]> {
  (...args: Args): void;
  clear: VoidFunction;
  callAndClear: VoidFunction;
}

export const debounce: ScheduleCallback = (callback, wait) => {
  if (isServer) {
    return Object.assign(() => void 0, {
      clear: () => void 0,
      callAndClear: () => void 0,
    });
  }
  let timeoutId: number | undefined;
  let timeoutFn: (() => void) | undefined;
  const clear = () => clearTimeout(timeoutId);
  const callAndClear = () => {
    timeoutFn?.();
    clear();
  };
  if (getOwner()) onCleanup(clear);
  const debounced: typeof callback = (...args) => {
    if (timeoutId !== undefined) clear();
    timeoutFn = () => {
      callback(...args);
      timeoutFn = undefined;
    };
    timeoutId = setTimeout(timeoutFn, wait);
  };
  return Object.assign(debounced, { clear, callAndClear });
};
