import { Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import {
  Checkbox,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui';

import { SelectGameVersionProps } from '.';

export function SelectGameVersion<T extends string>(
  props: SelectGameVersionProps<T>,
) {
  const [local, others] = splitProps(props, ['advanced', 'class']);

  return (
    <div class='flex gap-2'>
      <Select
        class={cn('w-full', local.class)}
        itemComponent={(props) => (
          <SelectItem item={props.item}>{props.item.rawValue}</SelectItem>
        )}
        {...others}
      >
        <SelectTrigger>
          <SelectValue<T>>{(state) => state.selectedOption()}</SelectValue>
        </SelectTrigger>
        <SelectContent />
      </Select>

      <Show when={local.advanced}>
        <div class='flex min-w-max items-center gap-2'>
          <Checkbox id='advanced-game-version' />
          <Label for='advanced-game-version'>Include snapshots</Label>
        </div>
      </Show>
    </div>
  );
}
