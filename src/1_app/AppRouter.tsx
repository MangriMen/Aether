import { Route, Router } from '@solidjs/router';
import type { Component } from 'solid-js';

import { AppRoot } from './AppRoot';
import { ContentPage } from '@/pages/content';
import { InstanceSettingsDialog } from '@/widgets/instance-settings-dialog';
import { HomePage } from '@/pages/home';
import { InstancePage } from '@/pages/instance';
import { SettingsPage } from '@/pages/settings';

export const AppRouter: Component = () => {
  return (
    <Router root={AppRoot}>
      <Route path='/' component={HomePage}>
        <Route />
        <Route
          path='/instance-settings/:id'
          component={(props) => <InstanceSettingsDialog id={props.params.id} />}
        />
      </Route>
      <Route path='/content' component={ContentPage} />
      <Route path='/instances/:id' component={InstancePage}>
        <Route />
        <Route
          path='settings'
          component={(props) => <InstanceSettingsDialog id={props.params.id} />}
        />
      </Route>
      <Route path='/settings' component={SettingsPage} />
    </Router>
  );
};
