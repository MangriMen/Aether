// eslint-disable-next-line import/named
import type { RouteSectionProps } from '@solidjs/router';
import type { Component } from 'solid-js';
import { createEffect, onCleanup, onMount } from 'solid-js';

import { Toaster } from '@/shared/ui';

import { initializeInstanceResource } from '@/entities/instance';

import ThemeObserver from './ThemeObserver';

import { ColorModeObserver, WindowObserver } from '.';

const RIGHT_CLICK_EXCLUDE_TAGS = new Set(['INPUT', 'SELECT', 'TEXTAREA']);

export const AppRoot: Component<RouteSectionProps> = (props) => {
  const preventFn = (e: MouseEvent) => {
    if (!(e.target instanceof HTMLElement)) {
      return;
    }

    if (RIGHT_CLICK_EXCLUDE_TAGS.has(e.target.tagName)) {
      return;
    }

    e.preventDefault();
  };
  createEffect(() => document.body.addEventListener('contextmenu', preventFn));

  onCleanup(() => document.body.removeEventListener('contextmenu', preventFn));

  onMount(() => {
    initializeInstanceResource();
  });

  return (
    <ColorModeObserver {...props}>
      <ThemeObserver>
        {props.children}
        <Toaster />
        <WindowObserver />
      </ThemeObserver>
    </ColorModeObserver>
  );
};
