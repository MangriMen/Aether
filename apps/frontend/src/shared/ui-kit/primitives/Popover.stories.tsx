import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { fn } from 'storybook/test';

import {
  createArgSignal,
  gutterArgType,
  placementArgType,
} from '../model/storyArgs';
import { Button } from './Button';
import { Popover, PopoverContent, PopoverTrigger } from './Popover';

const meta: Meta = {
  title: 'ui-kit/Popover',
  component: Popover,
  tags: ['autodocs'],
  args: { open: false, modal: false, gutter: 4, onOpenChange: fn() },
  argTypes: {
    open: { control: 'boolean' },
    modal: { control: 'boolean' },
    placement: placementArgType,
    gutter: gutterArgType,
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => {
    const [open, setOpen] = createArgSignal(() => Boolean(args['open']), false);

    return (
      <Popover
        {...args}
        open={open()}
        onOpenChange={(value) => {
          setOpen(value);
          args['onOpenChange']?.(value);
        }}
      >
        <PopoverTrigger as={Button<'button'>}>Trigger me</PopoverTrigger>
        <PopoverContent>
          A UI toolkit for building accessible web apps and design systems with
          SolidJS.
        </PopoverContent>
      </Popover>
    );
  },
};
