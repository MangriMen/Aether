import { Route, Router } from '@solidjs/router';
import type { Component } from 'solid-js';
import { lazy } from 'solid-js';

import { RunningInstancesProvider } from '@/entities/instances';

import { AppLayout } from '@/widgets/layouts/app-layout';
import { MainLayout } from '@/widgets/layouts/main-layout';

import { initializeApp } from './lib';
import { AppRoot } from './ui';

const HomePage = lazy(() =>
  import('@/pages/home').then((m) => ({ default: m.HomePage })),
);
const InstancePage = lazy(() =>
  import('@/pages/instance').then((m) => ({ default: m.InstancePage })),
);
const SettingsPage = lazy(() =>
  import('@/pages/settings').then((m) => ({ default: m.SettingsPage })),
);

export const AppRouter: Component = () => {
  return (
    <Router root={AppRoot} rootPreload={initializeApp}>
      <Route path='*' component={AppLayout}>
        <Route path='/' component={MainLayout}>
          <Route path='/' component={RunningInstancesProvider}>
            <Route path='/' component={HomePage} />
            <Route path='/settings' component={SettingsPage} />
            <Route path='/instances'>
              <Route path=':id/*' component={InstancePage} />
            </Route>
          </Route>
        </Route>
      </Route>
    </Router>
  );
};
