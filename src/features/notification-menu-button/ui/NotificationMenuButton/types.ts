import { PolymorphicProps } from '@kobalte/core';
import { ValidComponent } from 'solid-js';

import { IconButtonProps } from '@/shared/ui';

export type NotificationMenuButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>>;
