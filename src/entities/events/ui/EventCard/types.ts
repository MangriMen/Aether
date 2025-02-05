import type { ComponentProps } from 'solid-js';
import type { LoadingPayload } from '../../model';

export type EventCardProps = ComponentProps<'div'> & {
  payload: LoadingPayload;
};
