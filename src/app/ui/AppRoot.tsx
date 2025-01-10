// eslint-disable-next-line import/named
import { RouteSectionProps } from '@solidjs/router';
import { Component } from 'solid-js';

import { Toaster } from '@/shared/ui';

import { ColorModeObserver, WindowObserver } from '.';

export const AppRoot: Component<RouteSectionProps> = (props) => {
  return (
    <ColorModeObserver {...props}>
      {props.children}
      <Toaster />
      <WindowObserver />
    </ColorModeObserver>
  );
};
