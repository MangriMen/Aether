// eslint-disable-next-line import/named
import { RouteSectionProps } from '@solidjs/router';
import { Component, createEffect, onCleanup } from 'solid-js';

import { Toaster } from '@/shared/ui';

import { ColorModeObserver, WindowObserver } from '.';

export const AppRoot: Component<RouteSectionProps> = (props) => {
  const preventFn = (e: MouseEvent) => e.preventDefault();

  createEffect(() => document.body.addEventListener('contextmenu', preventFn));

  onCleanup(() => document.body.removeEventListener('contextmenu', preventFn));

  return (
    <ColorModeObserver {...props}>
      {props.children}
      <Toaster />
      <WindowObserver />
    </ColorModeObserver>
  );
};
