import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { fn } from 'storybook/test';

import {
  Slider,
  SliderFill,
  SliderLabel,
  SliderThumb,
  SliderTrack,
  SliderValueLabel,
} from './Slider';

const meta: Meta = {
  title: 'ui-kit/Slider',
  component: Slider,
  tags: ['autodocs'],
  args: {
    minValue: 10,
    maxValue: 2000,
    step: 1,
    disabled: false,
    onChange: fn(),
  },
  argTypes: {
    minValue: { control: 'number' },
    maxValue: { control: 'number' },
    step: { control: { type: 'number', min: 1 } },
    disabled: { control: 'boolean' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Range: Story = {
  args: { defaultValue: [20, 500] },
  render: (args) => (
    <Slider
      {...args}
      getValueLabel={(params) => `$${params.values[0]} - $${params.values[1]}`}
      class='w-75 space-y-3'
    >
      <div class='flex w-full justify-between'>
        <SliderLabel>Money</SliderLabel>
        <SliderValueLabel />
      </div>
      <SliderTrack>
        <SliderFill />
        <SliderThumb />
        <SliderThumb />
      </SliderTrack>
    </Slider>
  ),
};

export const Single: Story = {
  args: { minValue: 0, maxValue: 100, defaultValue: [40] },
  render: (args) => (
    <Slider {...args} class='w-75 space-y-3'>
      <div class='flex w-full justify-between'>
        <SliderLabel>Volume</SliderLabel>
        <SliderValueLabel />
      </div>
      <SliderTrack>
        <SliderFill />
        <SliderThumb />
      </SliderTrack>
    </Slider>
  ),
};
