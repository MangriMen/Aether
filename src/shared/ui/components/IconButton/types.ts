import { PolymorphicProps } from '@kobalte/core';
import { ValidComponent } from 'solid-js';

import { ButtonProps } from '../Button';

export type IconButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, ButtonProps<T>>;
