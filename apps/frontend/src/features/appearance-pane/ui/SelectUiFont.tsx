import { splitProps } from 'solid-js';

import type { UiFont } from '@/features/appearance';
import type { SelectRootProps, Option } from '@/shared/ui-kit';

import {
  Select,
  SelectContent,
  SelectErrorMessage,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

type SelectUiFontProps<Opt extends Option<UiFont> = Option<UiFont>> =
  SelectRootProps<Opt, never, 'div'> & {
    multiple?: false;
    errorMessage?: string;
  };

const SelectUiFont = <T extends Option<UiFont> = Option<UiFont>>(
  props: SelectUiFontProps<T>,
) => {
  const [local, others] = splitProps(props, ['errorMessage', 'class']);

  return (
    <Select
      class={cn('gap-2 flex flex-col', local.class)}
      validationState={local.errorMessage ? 'invalid' : 'valid'}
      optionTextValue='name'
      optionValue='value'
      itemComponent={(props) => (
        <SelectItem item={props.item}>{props.item.textValue}</SelectItem>
      )}
      {...others}
    >
      <SelectTrigger>
        <SelectValue<T>>{(state) => state.selectedOption()?.name}</SelectValue>
      </SelectTrigger>
      <SelectContent />
      <SelectErrorMessage>{local.errorMessage}</SelectErrorMessage>
    </Select>
  );
};

export default SelectUiFont;
