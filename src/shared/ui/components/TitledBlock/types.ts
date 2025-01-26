import type { ComponentProps } from 'solid-js';

export type TitledBlockProps = ComponentProps<'div'> & {
  title?: string;
};
