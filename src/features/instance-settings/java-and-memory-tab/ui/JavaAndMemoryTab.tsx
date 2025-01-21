import { Component, ComponentProps, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { InstanceSettingsTabProps } from '@/entities/instance';

import CustomMemory from './CustomMemory';
import CustomTextField from './CustomTextField';

export type JavaAndMemoryTabProps = ComponentProps<'div'> &
  InstanceSettingsTabProps;

const JavaAndMemoryTab: Component<JavaAndMemoryTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const handleChangeMemory = (value: number) => {
    console.log('Memory: ', value);
  };

  const handleChangeArguments = (value: string) => {
    console.log('Arguments: ', value);
  };

  const handleChangeEnvironmentVariables = (value: string) => {
    console.log('Environment variables: ', value);
  };

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <CustomMemory
        defaultValue={local.instance.memory?.maximum}
        onChange={handleChangeMemory}
      />
      <CustomTextField
        fieldLabel='Java arguments'
        label='Custom arguments'
        placeholder='Enter arguments'
        inputProps={{
          type: 'text',
          onBlur: (value) => handleChangeArguments(value.target.value),
        }}
      />
      <CustomTextField
        fieldLabel='Environment arguments'
        label='Custom environment variables'
        placeholder='Enter environment variables'
        inputProps={{
          type: 'text',
          onBlur: (value) =>
            handleChangeEnvironmentVariables(value.target.value),
        }}
      />
    </div>
  );
};

export default JavaAndMemoryTab;
