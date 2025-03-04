import { Route, Router } from '@solidjs/router';
import type { Component } from 'solid-js';
import { lazy } from 'solid-js';

import { RunningInstancesProvider } from '@/entities/instances';

import { MainLayout } from '@/app/layouts/MainLayout';

import { AppRoot } from './AppRoot';

const HomePage = lazy(() =>
  import('@/pages/home').then((m) => ({ default: m.HomePage })),
);
const InstancePage = lazy(() =>
  import('@/pages/instance').then((m) => ({ default: m.InstancePage })),
);

const SettingsPage = lazy(() =>
  import('@/pages/settings').then((m) => ({ default: m.SettingsPage })),
);

const InstanceSettingsDialog = lazy(() =>
  import('@/widgets/instance-settings-dialog').then((m) => ({
    default: m.InstanceSettingsDialog,
  })),
);

export const AppRouter: Component = () => {
  return (
    <Router root={AppRoot}>
      <Route path='/' component={MainLayout}>
        <Route path='/' component={RunningInstancesProvider}>
          <Route path='/' component={HomePage}>
            <Route />
            <Route
              path='/instance-settings/:id'
              component={(props) => (
                <InstanceSettingsDialog id={props.params.id} />
              )}
            />
          </Route>
          <Route path='/instances/:id' component={InstancePage}>
            <Route />
            <Route
              path='settings'
              component={(props) => (
                <InstanceSettingsDialog id={props.params.id} />
              )}
            />
          </Route>
          <Route path='/settings' component={SettingsPage} />
        </Route>
      </Route>
    </Router>
  );
};
