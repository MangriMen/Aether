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
