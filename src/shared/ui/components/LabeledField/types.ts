import type { ComponentProps, JSX } from 'solid-js';

export type FieldProps = ComponentProps<'div'> & {
  label?: string | JSX.Element;
};
