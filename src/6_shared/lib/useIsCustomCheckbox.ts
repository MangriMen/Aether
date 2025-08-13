import type { Accessor } from 'solid-js';
import { createEffect, createSignal } from 'solid-js';

export interface UseIsCustomCheckboxProps {
  isCustom: Accessor<boolean>;
  onChange: (isCustom: boolean) => void;
}

export const useIsCustomCheckbox = ({
  isCustom,
  onChange,
}: UseIsCustomCheckboxProps): [
  get: Accessor<boolean>,
  set: (value: boolean) => void,
] => {
  const [_isCustom, _setIsCustom] = createSignal(false);

  createEffect(() => {
    if (isCustom()) {
      _setIsCustom(true);
    }
  });

  const handleChangeIsCustom = (isChecked: boolean) => {
    _setIsCustom(isChecked);
    onChange?.(isChecked);
  };

  return [isCustom, handleChangeIsCustom];
};
