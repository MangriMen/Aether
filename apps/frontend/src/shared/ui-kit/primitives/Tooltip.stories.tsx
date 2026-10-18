import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { splitProps } from 'solid-js';
import { fn } from 'storybook/test';

import {
  createArgSignal,
  gutterArgType,
  placementArgType,
} from '../model/storyArgs';
import { Button } from './Button';
import { Tooltip, TooltipContent, TooltipTrigger } from './Tooltip';

const meta: Meta = {
  title: 'ui-kit/Tooltip',
  component: Tooltip,
  tags: ['autodocs'],
  args: {
    content: 'Tooltip content',
    open: false,
    disabled: false,
    placement: 'top',
    gutter: 4,
    openDelay: 700,
    closeDelay: 300,
    onOpenChange: fn(),
  },
  argTypes: {
    content: { control: 'text' },
    open: { control: 'boolean' },
    disabled: { control: 'boolean' },
    placement: placementArgType,
    gutter: gutterArgType,
    openDelay: { control: { type: 'number', min: 0, step: 100 } },
    closeDelay: { control: { type: 'number', min: 0, step: 100 } },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => {
    const [local, rest] = splitProps(args, ['content']);
    const [open, setOpen] = createArgSignal(() => Boolean(args['open']), false);

    return (
      <Tooltip
        {...rest}
        open={open()}
        onOpenChange={(value) => {
          setOpen(value);
          args['onOpenChange']?.(value);
        }}
      >
        <TooltipTrigger as={Button<'button'>} variant='outline'>
          Trigger
        </TooltipTrigger>
        <TooltipContent>
          <p>{local['content']}</p>
        </TooltipContent>
      </Tooltip>
    );
  },
};
