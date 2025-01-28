import type { ComponentProps } from 'solid-js';


import type { LoadingPayload } from '@/entities/minecraft';

export type EventCardProps = ComponentProps<'div'> & {
  payload: LoadingPayload;
};
