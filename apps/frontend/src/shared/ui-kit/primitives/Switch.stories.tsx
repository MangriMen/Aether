import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { splitProps } from 'solid-js';
import { fn } from 'storybook/test';

import { createArgSignal } from '../model/storyArgs';
import { Switch, SwitchControl, SwitchLabel, SwitchThumb } from './Switch';

const meta: Meta = {
  title: 'ui-kit/Switch',
  component: Switch,
  tags: ['autodocs'],
  args: { label: 'Airplane Mode', disabled: false, onChange: fn() },
  argTypes: {
    label: { control: 'text' },
    checked: { control: 'boolean' },
    disabled: { control: 'boolean' },
    required: { control: 'boolean' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => {
    const [local, rest] = splitProps(args, ['label']);
    const [checked, setChecked] = createArgSignal(
      () => Boolean(args['checked']),
      false,
    );

    return (
      <Switch
        {...rest}
        checked={checked()}
        onChange={(next) => {
          setChecked(next);
          args['onChange']?.(next);
        }}
        class='space-x-2 flex items-center'
      >
        <SwitchControl>
          <SwitchThumb />
        </SwitchControl>
        <SwitchLabel>{local['label']}</SwitchLabel>
      </Switch>
    );
  },
};
