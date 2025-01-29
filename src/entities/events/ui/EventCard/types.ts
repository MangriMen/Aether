import type { ComponentProps } from 'solid-js';

import type { LoadingPayload } from '@/entities/minecrafts';

export type EventCardProps = ComponentProps<'div'> & {
  payload: LoadingPayload;
};
