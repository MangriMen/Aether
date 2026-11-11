import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { fn } from 'storybook/test';

import {
  Combobox,
  ComboboxContent,
  ComboboxControl,
  ComboboxInput,
  ComboboxItem,
  ComboboxTrigger,
} from './Combobox';

const meta: Meta = {
  title: 'ui-kit/Combobox',
  component: Combobox,
  tags: ['autodocs'],
  args: {
    options: ['Apple', 'Banana', 'Blueberry', 'Grapes', 'Pineapple'],
    disabled: false,
    onChange: fn(),
  },
  argTypes: {
    options: { control: 'object' },
    disabled: { control: 'boolean' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => (
    <Combobox<string>
      {...args}
      options={args['options']}
      itemComponent={(props) => (
        <ComboboxItem item={props.item}>{props.item.rawValue}</ComboboxItem>
      )}
    >
      <ComboboxControl aria-label='Fruit' class='w-45'>
        <ComboboxInput placeholder='Pick or type…' />
        <ComboboxTrigger />
      </ComboboxControl>
      <ComboboxContent />
    </Combobox>
  ),
};
