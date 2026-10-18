import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { fn } from 'storybook/test';

import { Checkbox } from './Checkbox';
import { Label } from './Label';

const meta: Meta = {
  title: 'ui-kit/Checkbox',
  component: Checkbox,
  tags: ['autodocs'],
  args: {
    label: 'Accept terms and conditions',
    disabled: false,
    indeterminate: false,
    onChange: fn(),
  },
  argTypes: {
    label: { control: 'text' },
    checked: { control: 'boolean' },
    indeterminate: { control: 'boolean' },
    disabled: { control: 'boolean' },
    required: { control: 'boolean' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => <Checkbox {...args} />,
};

export const WithDescription: Story = {
  render: (args) => (
    <div class='space-x-2 flex items-start'>
      <Checkbox {...args} label={undefined} id='terms1' />
      <div class='gap-1.5 grid leading-none'>
        <Label for='terms1-input'>{args['label']}</Label>
        <p class='text-sm text-muted-foreground'>
          You agree to our Terms of Service and Privacy Policy.
        </p>
      </div>
    </div>
  ),
};
