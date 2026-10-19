import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { createSignal } from 'solid-js';
import { fn } from 'storybook/test';

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './Select';

const meta: Meta = {
  title: 'ui-kit/Select',
  component: Select,
  tags: ['autodocs'],
  args: {
    options: ['Apple', 'Banana', 'Blueberry', 'Grapes', 'Pineapple'],
    placeholder: 'Select a fruit…',
    disabled: false,
    required: false,
    onChange: fn(),
  },
  argTypes: {
    options: { control: 'object' },
    placeholder: { control: 'text' },
    disabled: { control: 'boolean' },
    required: { control: 'boolean' },
    disallowEmptySelection: { control: 'boolean' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => {
    const [value, setValue] = createSignal('');

    return (
      <Select
        {...args}
        options={args['options']}
        value={value()}
        onChange={(next: string | null) => {
          setValue(next ?? '');
          args['onChange']?.(next);
        }}
        itemComponent={(props) => (
          <SelectItem item={props.item}>{props.item.rawValue}</SelectItem>
        )}
      >
        <SelectTrigger aria-label='Fruit' class='w-45'>
          <SelectValue<string>>{(state) => state.selectedOption()}</SelectValue>
        </SelectTrigger>
        <SelectContent />
      </Select>
    );
  },
};

type VersionOption = { id: string };

const makeVersions = (count: number, offset = 0): VersionOption[] =>
  Array.from({ length: count }, (_, i) => ({ id: `v1.0.${offset + i}` }));

export const Virtualized: Story = {
  render: () => {
    const options = makeVersions(5000);

    return (
      <Select<VersionOption>
        virtualized
        options={options}
        optionValue='id'
        optionTextValue='id'
      >
        <SelectTrigger aria-label='Version' class='w-45'>
          <SelectValue<VersionOption>>
            {(state) => state.selectedOption()?.id}
          </SelectValue>
        </SelectTrigger>
        <SelectContent
          class='h-42.5'
          virtualized
          options={options}
          optionValue='id'
          itemComponent={(props) => (
            <SelectItem item={props.item} style={props.style}>
              {props.item.textValue}
            </SelectItem>
          )}
        />
      </Select>
    );
  },
};

export const VirtualizedChangingOptions: Story = {
  render: () => {
    const [options, setOptions] = createSignal(makeVersions(500));

    return (
      <div class='gap-2 flex flex-col'>
        <div class='gap-2 flex'>
          <button
            type='button'
            class='px-2 border'
            onClick={() =>
              setOptions((prev) => [...prev, ...makeVersions(100, prev.length)])
            }
          >
            Add 100
          </button>
          <button
            type='button'
            class='px-2 border'
            onClick={() => setOptions((prev) => prev.slice(0, -100))}
          >
            Remove 100
          </button>
          <span>{options().length} options</span>
        </div>
        <Select<VersionOption>
          virtualized
          options={options()}
          optionValue='id'
          optionTextValue='id'
        >
          <SelectTrigger aria-label='Version' class='w-45'>
            <SelectValue<VersionOption>>
              {(state) => state.selectedOption()?.id}
            </SelectValue>
          </SelectTrigger>
          <SelectContent
            class='h-42.5'
            virtualized
            options={options()}
            optionValue='id'
            itemComponent={(props) => (
              <SelectItem item={props.item} style={props.style}>
                {props.item.textValue}
              </SelectItem>
            )}
          />
        </Select>
      </div>
    );
  },
};
