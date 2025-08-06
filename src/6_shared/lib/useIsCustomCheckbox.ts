import type { Accessor } from 'solid-js';
import { createEffect, createSignal, untrack } from 'solid-js';

export interface UseIsCustomCheckboxProps<T> {
  value: Accessor<T>;
  resetValue: Accessor<T>;
  onOverrideValue?: (value: T) => void;
}

export const useIsCustomCheckbox = <T>({
  value,
  resetValue,
  onOverrideValue,
}: UseIsCustomCheckboxProps<T>): [
  get: Accessor<boolean>,
  set: (value: boolean) => void,
] => {
  const [isCustom, setIsCustom] = createSignal(false);

  createEffect(() => {
    if (value()) {
      setIsCustom(true);
    }
  });

  const handleChangeIsCustom = (isChecked: boolean) => {
    setIsCustom(isChecked);
    untrack(() => {
      onOverrideValue?.(isChecked ? value() : resetValue());
    });
  };

  return [isCustom, handleChangeIsCustom];
};
