import { type Accessor } from 'solid-js';

export const createOverrideHandler = <T, F = null>(
  enabledValue: Accessor<T>,
  disabledValue: Accessor<F>,
  onChange: (value: T | F) => void,
) => {
  return (isEnabled: boolean) =>
    onChange(isEnabled ? enabledValue() : disabledValue());
};
