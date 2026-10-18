import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import IconMdiChevronDown from '~icons/mdi/chevron-down';
import { fn } from 'storybook/test';

import { createArgSignal } from '../model/storyArgs';
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from './Collapsible';

const meta: Meta = {
  title: 'ui-kit/Collapsible',
  component: Collapsible,
  tags: ['autodocs'],
  args: { open: false, disabled: false, onOpenChange: fn() },
  argTypes: {
    open: { control: 'boolean' },
    disabled: { control: 'boolean' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => {
    const [open, setOpen] = createArgSignal(() => Boolean(args['open']), false);

    return (
      <Collapsible
        {...args}
        open={open()}
        onOpenChange={(value) => {
          setOpen(value);
          args['onOpenChange']?.(value);
        }}
        class='w-75'
      >
        <CollapsibleTrigger class='rounded-md p-3 flex w-full items-center justify-between border'>
          <span>What is Kobalte ?</span>
          <IconMdiChevronDown />
        </CollapsibleTrigger>
        <CollapsibleContent class='mt-2 rounded-md p-3 border'>
          Kobalte is a UI toolkit for building accessible web apps and design
          systems with SolidJS. It provides a set of low-level UI components and
          primitives which can be the foundation for your design system
          implementation.
        </CollapsibleContent>
      </Collapsible>
    );
  },
};
