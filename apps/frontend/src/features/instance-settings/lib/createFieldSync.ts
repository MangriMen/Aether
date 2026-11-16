import type { Accessor } from 'solid-js';

import { createEffect, createMemo, on, untrack } from 'solid-js';

import { logError } from '@/shared/log';

/**
 * Puts one stored setting into its field whenever that stored value changes, and returns a
 * function that puts it there again on demand.
 *
 * The stored settings are rebuilt on every refresh, so only a change of this one value may reset
 * the field; otherwise saving one setting would overwrite another that the user is still typing.
 * `apply` resets the field with the value as its initial input, so `isDirty` tells whether the
 * field differs from what is stored and a blur without a change sends nothing.
 */
export const createFieldSync = <T>(
  stored: Accessor<T | undefined>,
  apply: (value: T) => void,
) => {
  const applyDefined = (value: T | undefined) => {
    if (value !== undefined) {
      apply(value);
    }
  };

  createEffect(on(createMemo(stored), applyDefined));

  return () => applyDefined(untrack(stored));
};

/**
 * Puts the stored values back into the fields of `values`, and only those: other fields may hold
 * input the user has not finished yet.
 */
export const reapplyFields = <K extends string>(
  reappliers: Partial<Record<K, () => void>>,
  values: Partial<Record<K, unknown>>,
) => {
  for (const key of Object.keys(values) as K[]) {
    reappliers[key]?.();
  }
};

/**
 * A failed save leaves the stored value as it was, so `createFieldSync` does not fire and the field
 * would keep showing the value that was not saved; `revert` puts the stored one back.
 */
export const revertOnFailure = (
  saving: Promise<unknown> | void,
  revert: () => void,
) => {
  Promise.resolve(saving).catch((err: unknown) => {
    logError(err);
    revert();
  });
};
