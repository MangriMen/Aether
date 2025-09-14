import { type Accessor, createEffect } from 'solid-js';

export const useProgressMenuShowActions = (
  payloadsCount: Accessor<number>,
  onZeroCount: () => void,
  onPrevWasZeroCount: () => void,
) => {
  createEffect((prevCount) => {
    const currCount = payloadsCount();

    if (!currCount) {
      onZeroCount();
    } else if (prevCount === 0 && currCount) {
      onPrevWasZeroCount();
    }

    return currCount;
  }, 0);
};
