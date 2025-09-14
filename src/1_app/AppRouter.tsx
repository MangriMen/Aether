import type { Component } from 'solid-js';

import { Route, Router } from '@solidjs/router';

import { ContentPage } from '@/pages/content';
import { HomePage } from '@/pages/home';
import { InstancePage } from '@/pages/instance';
import { SettingsPage } from '@/pages/settings';
import { InstanceSettingsDialog } from '@/widgets/instance-settings-dialog';

import { AppRoot } from './AppRoot';

export const AppRouter: Component = () => {
  return (
    <Router root={AppRoot}>
      <Route component={HomePage} path='/'>
        <Route />
        <Route
          component={(props) => (
            <InstanceSettingsDialog instanceId={props.params.id} />
          )}
          path='/instance-settings/:id'
        />
      </Route>
      <Route component={ContentPage} path='/content' />
      <Route component={InstancePage} path='/instances/:id'>
        <Route />
        <Route
          component={(props) => (
            <InstanceSettingsDialog instanceId={props.params.id} />
          )}
          path='settings'
        />
      </Route>
      <Route component={SettingsPage} path='/settings' />
    </Router>
  );
};
