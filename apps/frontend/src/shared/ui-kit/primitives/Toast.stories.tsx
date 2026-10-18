import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { For } from 'solid-js';

import type { ShowToastParams } from './Toast';

import { Button } from './Button';
import { showToast, Toaster } from './Toast';

const VARIANTS = [
  'default',
  'destructive',
  'success',
  'warning',
  'error',
] as const;

const meta: Meta = {
  title: 'ui-kit/Toast',
  tags: ['autodocs'],
  decorators: [
    (Story) => (
      <>
        <Story />
        <Toaster />
      </>
    ),
  ],
  args: {
    title: 'Event created',
    description: 'Monday, Jan 3rd',
    variant: 'default',
    duration: 5000,
    persistent: false,
  },
  argTypes: {
    title: { control: 'text' },
    description: { control: 'text' },
    variant: { control: 'select', options: VARIANTS },
    duration: { control: { type: 'number', min: 0, step: 500 } },
    persistent: { control: 'boolean' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Custom: Story = {
  render: (args) => (
    <Button
      onClick={() =>
        showToast({
          title: args['title'],
          description: args['description'],
          variant: args['variant'],
          duration: args['duration'],
          persistent: args['persistent'],
        })
      }
    >
      Show toast
    </Button>
  ),
};

export const AllVariants: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <div class='gap-2 flex flex-wrap'>
      <For each={VARIANTS}>
        {(variant) => (
          <Button
            variant='outline'
            class='capitalize'
            onClick={() =>
              showToast({
                title: variant,
                description: 'Monday, Jan 3rd',
                variant,
              })
            }
          >
            {variant}
          </Button>
        )}
      </For>
    </div>
  ),
};

const toastStory = (
  label: string,
  params: ShowToastParams,
  buttonVariant: 'default' | 'destructive' | 'outline' = 'outline',
): Story => ({
  parameters: { controls: { disable: true } },
  render: () => (
    <Button variant={buttonVariant} onClick={() => showToast(params)}>
      {label}
    </Button>
  ),
});

export const Default = toastStory(
  'Default',
  { title: 'Event created', description: 'Monday, Jan 3rd' },
  'default',
);
export const Destructive = toastStory(
  'Destructive',
  {
    title: 'Event deleted',
    description: 'Monday, Jan 3rd',
    variant: 'destructive',
  },
  'destructive',
);
export const Success = toastStory('Success', {
  title: 'SUCCESS!',
  description: 'Monday, Jan 3rd',
  variant: 'success',
});
export const Warning = toastStory('Warning', {
  title: 'WARNING!',
  description: 'Monday, Jan 3rd',
  variant: 'warning',
});
export const ErrorToast = toastStory('Error', {
  title: 'ERROR!',
  description: 'Monday, Jan 3rd',
  variant: 'error',
});
