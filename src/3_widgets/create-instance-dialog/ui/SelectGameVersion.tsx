import { splitProps } from 'solid-js';

import type { Version } from '@/entities/minecraft';
import type { SelectRootProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import {
  Select,
  SelectContent,
  SelectErrorMessage,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui';

export type SelectGameVersionProps<Option extends Version = Version> = {
  errorMessage?: string;
  multiple?: false;
} & SelectRootProps<Option, never, 'div'>;

export function SelectGameVersion<T extends Version = Version>(
  props: SelectGameVersionProps<T>,
) {
  const [local, others] = splitProps(props, ['value', 'errorMessage', 'class']);

  return (
    <Select
      class={cn('flex flex-col gap-2 w-full', local.class)}
      optionTextValue={'id'}
      optionValue={'id'}
      validationState={local.errorMessage ? 'invalid' : 'valid'}
      value={local.value}
      virtualized
      {...others}
    >
      <SelectTrigger>
        <SelectValue<T>>
          {(state) => state.selectedOption()?.id ?? local.value?.id}
        </SelectValue>
      </SelectTrigger>
      <SelectContent
        class='h-[170px]'
        itemComponent={(props) => (
          <SelectItem item={props.item} style={props.style}>
            {props.item.textValue}
          </SelectItem>
        )}
        options={others.options}
        optionValue={'id'}
        virtualized
      />
      <SelectErrorMessage>{local.errorMessage}</SelectErrorMessage>
    </Select>
  );
}
