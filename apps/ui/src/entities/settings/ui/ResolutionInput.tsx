import { splitProps, type Component } from 'solid-js';

import { cn } from '@/shared/lib';
import { CombinedTextField, type CombinedTextFieldProps } from '@/shared/ui';

export type ResolutionInputProps = CombinedTextFieldProps;

export const ResolutionInput: Component<ResolutionInputProps> = (props) => {
  const [local, others] = splitProps(props, ['inputProps', 'class']);

  return (
    <CombinedTextField
      class={cn('w-[6ch]', local.class)}
      inputProps={{
        type: 'text',
        class:
          'border-none bg-transparent focus:bg-transparent grow focus-visible:ring-none focus-visible:ring-0 focus-visible:shadow-none text-center px-0',
        maxLength: 5,
        ...local.inputProps,
      }}
      {...others}
    />
  );
};
