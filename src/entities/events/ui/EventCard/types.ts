import { ComponentProps } from 'solid-js';

// eslint-disable-next-line boundaries/element-types
import { LoadingPayload } from '@/entities/minecraft';

export type EventCardProps = ComponentProps<'div'> & {
  payload: LoadingPayload;
};
