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

export const Disabled: Story = {
  args: { disabled: true },
  render: Default.render,
};

const MANY_OPTIONS = Array.from(
  { length: 1000 },
  (_, index) => `Option ${index + 1}`,
);

/** Only the rows in view are drawn: the root and the content are both `virtualized`. */
export const Virtualized: Story = {
  render: (args) => (
    <Combobox<string>
      {...args}
      virtualized
      options={MANY_OPTIONS}
      itemComponent={(props) => (
        <ComboboxItem item={props.item}>{props.item.rawValue}</ComboboxItem>
      )}
    >
      <ComboboxControl aria-label='Option' class='w-45'>
        <ComboboxInput placeholder='Pick or type…' />
        <ComboboxTrigger />
      </ComboboxControl>
      <ComboboxContent<string>
        virtualized
        class='max-h-72'
        itemComponent={(props) => (
          <ComboboxItem item={props.item} style={props.style}>
            {props.item.rawValue}
          </ComboboxItem>
        )}
      />
    </Combobox>
  ),
};
