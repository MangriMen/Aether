import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { fn } from 'storybook/test';

import { createArgSignal } from '../model/storyArgs';
import { Button } from './Button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from './Dialog';
import { TextField, TextFieldInput, TextFieldLabel } from './TextField';

const meta: Meta = {
  title: 'ui-kit/Dialog',
  component: Dialog,
  tags: ['autodocs'],
  args: { open: false, modal: true, onOpenChange: fn() },
  argTypes: {
    open: { control: 'boolean' },
    modal: { control: 'boolean' },
    preventScroll: { control: 'boolean' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => {
    const [open, setOpen] = createArgSignal(() => Boolean(args['open']), false);

    return (
      <Dialog
        {...args}
        open={open()}
        onOpenChange={(value) => {
          setOpen(value);
          args['onOpenChange']?.(value);
        }}
      >
        <DialogTrigger as={Button<'button'>}>Edit Profile</DialogTrigger>
        <DialogContent class='sm:max-w-106.25'>
          <DialogHeader>
            <DialogTitle>Edit profile</DialogTitle>
            <DialogDescription>
              Make changes to your profile here. Click save when you're done.
            </DialogDescription>
          </DialogHeader>
          <div class='gap-4 py-4 grid'>
            <TextField class='gap-4 grid grid-cols-4 items-center'>
              <TextFieldLabel class='text-right'>Name</TextFieldLabel>
              <TextFieldInput
                value='Pedro Duarte'
                class='col-span-3'
                type='text'
              />
            </TextField>
            <TextField class='gap-4 grid grid-cols-4 items-center'>
              <TextFieldLabel class='text-right'>Username</TextFieldLabel>
              <TextFieldInput
                value='@peduarte'
                class='col-span-3'
                type='text'
              />
            </TextField>
          </div>
          <DialogFooter>
            <Button type='submit'>Save changes</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    );
  },
};

/** Drag a side edge to change the width; double-click an edge to reset it. */
export const Resizable: Story = {
  render: () => (
    <Dialog>
      <DialogTrigger as={Button<'button'>}>Open resizable</DialogTrigger>
      <DialogContent resizable defaultWidth={480} minWidth={320}>
        <DialogHeader>
          <DialogTitle>Resizable</DialogTitle>
          <DialogDescription>
            Drag the left or right edge of the window.
          </DialogDescription>
        </DialogHeader>
      </DialogContent>
    </Dialog>
  ),
};
