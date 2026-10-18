import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { splitProps } from 'solid-js';
import { fn } from 'storybook/test';

import { createArgSignal } from '../model/storyArgs';
import { TextField, TextFieldInput, TextFieldLabel } from './TextField';

const meta: Meta = {
  title: 'ui-kit/TextField',
  component: TextField,
  tags: ['autodocs'],
  args: {
    label: 'Email',
    placeholder: 'Email',
    type: 'email',
    value: '',
    disabled: false,
    readOnly: false,
    required: false,
    onChange: fn(),
  },
  argTypes: {
    label: { control: 'text' },
    placeholder: { control: 'text' },
    type: {
      control: 'select',
      options: ['text', 'email', 'password', 'number', 'search', 'url', 'tel'],
    },
    value: { control: 'text' },
    validationState: { control: 'select', options: ['valid', 'invalid'] },
    disabled: { control: 'boolean' },
    readOnly: { control: 'boolean' },
    required: { control: 'boolean' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => {
    const [local, rest] = splitProps(args, ['label', 'placeholder', 'type']);
    const [value, setValue] = createArgSignal(
      () => String(args['value'] ?? ''),
      '',
    );

    return (
      <TextField
        {...rest}
        value={value()}
        onChange={(next: string) => {
          setValue(next);
          args['onChange']?.(next);
        }}
        class='max-w-sm gap-1.5 grid w-full items-center'
      >
        <TextFieldLabel>{local['label']}</TextFieldLabel>
        <TextFieldInput
          type={local['type']}
          placeholder={local['placeholder']}
        />
      </TextField>
    );
  },
};
