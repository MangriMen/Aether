import {
  createEffect,
  createMemo,
  createSignal,
  Show,
  splitProps,
} from 'solid-js';

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
        optionValue={'id'}
        optionTextValue={'id'}
        defaultValue={local.defaultValue}
        value={local.value}
        onChange={local.onChange}
        itemComponent={(props) => (
          <SelectItem item={props.item}>{props.item.rawValue?.id}</SelectItem>
        )}
        options={version_list()}
        {...others}
      >
        <SelectTrigger>
          <SelectValue<T>>{(state) => state.selectedOption()?.id}</SelectValue>
        </SelectTrigger>
        <SelectContent class='max-h-[148px] overflow-y-auto' />
      </Select>

      <Show when={local.advanced}>
        <div class='flex min-w-max items-center gap-2'>
          <Checkbox
            id='advanced-game-version'
            onChange={handleIncludeSnapshotsChange}
          />
          <Label for='advanced-game-version'>Include snapshots</Label>
        </div>
      </Show>
    </div>
  );
}
