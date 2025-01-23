import { Route, Router } from '@solidjs/router';
import { Component, lazy } from 'solid-js';

import { RunningInstancesProvider } from '@/entities/instance';

import { AppLayout } from '@/widgets/layouts/app-layout';
import { MainLayout } from '@/widgets/layouts/main-layout';

import { initializeApp } from './lib';
import { AppRoot } from './ui';

const HomePage = lazy(() =>
  import('@/pages/home-page').then((m) => ({ default: m.HomePage })),
);
const InstancePage = lazy(() =>
  import('@/pages/instance-page').then((m) => ({ default: m.InstancePage })),
);
const SettingsPage = lazy(() =>
  import('@/pages/settings-page').then((m) => ({ default: m.SettingsPage })),
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
