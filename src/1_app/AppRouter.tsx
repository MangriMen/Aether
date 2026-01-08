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
      <Route path='/' component={HomePage}>
        <Route />
        <Route
          path='/instance-settings/:id'
          component={(props) => (
            <InstanceSettingsDialog instanceId={props.params.id} />
          )}
        />
      </Route>
      <Route path='/content' component={ContentPage} />
      <Route path='/instances/:id' component={InstancePage}>
        <Route />
        <Route
          path='settings'
          component={(props) => (
            <InstanceSettingsDialog instanceId={props.params.id} />
          )}
        />
      </Route>
      <Route path='/settings/:tab?' component={SettingsPage}>
        <Route />
      </Route>
    </Router>
  );
};
