import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui';

import { Version } from '@/entities/minecraft';

import { SelectGameVersionProps } from '.';

export function SelectGameVersion<T extends Version = Version>(
  props: SelectGameVersionProps<T>,
) {
  const [local, others] = splitProps(props, ['value', 'class']);

  return (
    <Select
      class={cn('w-full', local.class)}
      value={local.value}
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
    </Select>
  );
}
