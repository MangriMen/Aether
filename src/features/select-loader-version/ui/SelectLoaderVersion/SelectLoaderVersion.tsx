import { Component, createSignal, For, Show, splitProps } from 'solid-js';

import { Option } from '@/shared/model';
import {
  Field,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  ToggleGroup,
  ToggleGroupItem,
} from '@/shared/ui';

import { SelectLoaderVersionProps } from '.';

const loaderVersionTypes: Option[] = [
  {
    name: 'Stable',
    value: 'stable',
  },
  {
    name: 'Latest',
    value: 'latest',
  },
  {
    name: 'Other',
    value: 'other',
  },
];

const specificVersions = ['48.48.48', '47.47.47', '46.46.46'];

export const SelectLoaderVersion: Component<SelectLoaderVersionProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['onChange']);

  const [value, setValue] = createSignal(loaderVersionTypes[0].value);

  const [specificVersion, setSpecificVersion] = createSignal<
    string | undefined
  >(undefined);

  return (
    <div {...others}>
      <Field label='Loader version'>
        <ToggleGroup
          class='justify-start'
          defaultValue={loaderVersionTypes[0].value}
          value={value()}
          onChange={(value) => {
            if (value) {
              setValue(value);

              if (value !== 'other') {
                local.onChange?.(value);
              }
            }
          }}
        >
          <For each={loaderVersionTypes}>
            {(versionType) => (
              <ToggleGroupItem value={versionType.value}>
                {versionType.name}
              </ToggleGroupItem>
            )}
          </For>
        </ToggleGroup>

        <Show when={value() === 'other'}>
          <Field label='Select version'>
            <Select
              class='w-full'
              placeholder='Select loader version'
              value={specificVersion()}
              onChange={(value) => {
                if (value) {
                  setSpecificVersion(value);
                  local.onChange?.(value);
                }
              }}
              options={specificVersions}
              itemComponent={(props) => (
                <SelectItem item={props.item}>{props.item.rawValue}</SelectItem>
              )}
            >
              <SelectTrigger>
                <SelectValue<string>>
                  {(state) => state.selectedOption()}
                </SelectValue>
              </SelectTrigger>
              <SelectContent />
            </Select>
          </Field>
        </Show>
      </Field>
    </div>
  );
};
