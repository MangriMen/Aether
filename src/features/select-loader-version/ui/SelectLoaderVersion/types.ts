import { ComponentProps } from 'solid-js';

export type SelectLoaderVersionProps = Omit<
  ComponentProps<'div'>,
  'onChange'
> & {
  onChange: (value: string) => void;
};
