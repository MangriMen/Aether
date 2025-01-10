import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import {
  Select,
  SelectContent,
  SelectErrorMessage,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui';

import { Version } from '@/entities/minecraft';

import { SelectGameVersionProps } from '.';

export function SelectGameVersion<T extends Version = Version>(
  props: SelectGameVersionProps<T>,
) {
  const [local, others] = splitProps(props, ['value', 'errorMessage', 'class']);

  return (
    <Select
      class={cn('flex flex-col gap-2 w-full', local.class)}
      value={local.value}
      validationState={local.errorMessage ? 'invalid' : 'valid'}
      optionValue={'id'}
      optionTextValue={'id'}
      itemComponent={(props) => (
        <SelectItem item={props.item}>{props.item.textValue}</SelectItem>
      )}
      {...others}
    >
      <SelectTrigger>
        <SelectValue<T>>
          {(state) => state.selectedOption()?.id ?? local.value?.id}
        </SelectValue>
      </SelectTrigger>
      <SelectContent class='max-h-[148px] overflow-y-auto' />
      <SelectErrorMessage>{local.errorMessage}</SelectErrorMessage>
    </Select>
  );
}
