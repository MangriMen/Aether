import type { PolymorphicProps } from '@kobalte/core';
import type { ValidComponent } from 'solid-js';

import type { IconButtonProps } from '@/shared/ui';

export type NotificationMenuButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>>;
