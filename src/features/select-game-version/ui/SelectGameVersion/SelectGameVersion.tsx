import { createEffect, createMemo, createSignal, splitProps } from 'solid-js';

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

import { Version, VersionType } from '@/entities/minecraft';

import { SelectGameVersionProps } from '.';

export function SelectGameVersion<T extends Version = Version>(
  props: SelectGameVersionProps<T>,
) {
  const [local, others] = splitProps(props, [
    'advanced',
    'defaultValue',
    'value',
    'onChange',
    'options',
    'class',
  ]);

  const [shouldIncludeSnapshots, setShouldIncludeSnapshots] =
    createSignal(false);

  const version_list = createMemo(
    () =>
      local.options.filter(
        (version) =>
          version.type === VersionType.Release ||
          (local.advanced &&
            shouldIncludeSnapshots() &&
            version.type === VersionType.Snapshot),
      ) ?? [],
  );

  const handleIncludeSnapshotsChange = (value: boolean) => {
    if (
      !value &&
      local.value !== null &&
      local.value !== undefined &&
      local.value.type === VersionType.Snapshot
    ) {
      local.onChange?.(null);
    }

    setShouldIncludeSnapshots(value);
  };

  createEffect(() => {
    if (!local.advanced) {
      setShouldIncludeSnapshots(false);
    }
  });

  return (
    <div class='flex gap-2'>
      <Select
        class={cn('w-full', local.class)}
        options={version_list()}
        optionValue={'id'}
        optionTextValue={'id'}
        defaultValue={local.defaultValue}
        value={local.value}
        onChange={local.onChange}
        itemComponent={(props) => (
          <SelectItem item={props.item}>{props.item.textValue}</SelectItem>
        )}
        {...others}
      >
        <SelectTrigger>
          <SelectValue<T>>{(state) => state.selectedOption()?.id}</SelectValue>
        </SelectTrigger>
        <SelectContent class='max-h-[148px] overflow-y-auto' />
      </Select>

      <div
        class={cn(
          'flex min-w-max items-center gap-2 animate-in fade-in-0 duration-300',
          {
            'invisible animate-out fade-out-0': !local.advanced,
          },
        )}
      >
        <Checkbox
          id='advanced-game-version'
          onChange={handleIncludeSnapshotsChange}
        />
        <Label for='advanced-game-version-input'>Include snapshots</Label>
      </div>
    </div>
  );
}
