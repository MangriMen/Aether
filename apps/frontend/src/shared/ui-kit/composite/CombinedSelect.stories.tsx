import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { fn } from 'storybook/test';

import { CombinedSelect } from './CombinedSelect';

type Named = { id: number; name: string };

const NAMED: Named[] = [
  { id: 1, name: 'Alpha' },
  { id: 2, name: 'Beta' },
  { id: 3, name: 'Gamma' },
];

const meta: Meta = {
  title: 'ui-kit/CombinedSelect',
  component: CombinedSelect,
  tags: ['autodocs'],
  args: {
    disabled: false,
    onChange: fn(),
  },
  argTypes: {
    disabled: { control: 'boolean' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => (
    <CombinedSelect<Named, never>
      {...args}
      options={NAMED}
      optionValue='id'
      optionTextValue='name'
      defaultValue={NAMED[0]}
    />
  ),
};

export const FunctionTextValue: Story = {
  render: (args) => (
    <CombinedSelect<Named, never>
      {...args}
      options={NAMED}
      optionValue='id'
      optionTextValue={(option) => `#${option.id} ${option.name}`}
      defaultValue={NAMED[1]}
    />
  ),
};

export const PrimitiveOptions: Story = {
  render: (args) => (
    <CombinedSelect<number | string, never>
      {...args}
      options={[0, 1, 2, 'three', 'four']}
      defaultValue={0}
    />
  ),
};

export const WithClass: Story = {
  render: (args) => (
    <CombinedSelect<Named, never>
      {...args}
      class='w-60'
      options={NAMED}
      optionValue='id'
      optionTextValue='name'
      defaultValue={NAMED[2]}
    />
  ),
};
