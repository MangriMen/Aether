import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { fn } from 'storybook/test';

import { orientationArgType } from '../model/storyArgs';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './Tabs';

const meta: Meta = {
  title: 'ui-kit/Tabs',
  component: Tabs,
  tags: ['autodocs'],
  args: {
    defaultValue: 'account',
    orientation: 'horizontal',
    activationMode: 'automatic',
    disabled: false,
    onChange: fn(),
  },
  argTypes: {
    defaultValue: { control: 'select', options: ['account', 'password'] },
    orientation: orientationArgType,
    activationMode: { control: 'select', options: ['automatic', 'manual'] },
    disabled: { control: 'boolean' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => (
    <Tabs {...args} class='w-100'>
      <TabsList class='grid w-full grid-cols-2'>
        <TabsTrigger value='account'>Account</TabsTrigger>
        <TabsTrigger value='password'>Password</TabsTrigger>
      </TabsList>
      <TabsContent value='account'>Account</TabsContent>
      <TabsContent value='password'>Password</TabsContent>
    </Tabs>
  ),
};
