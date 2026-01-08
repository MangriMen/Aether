import type { Component } from 'solid-js';

import { Route, Router } from '@solidjs/router';

import { ContentPage } from '@/pages/content';
import { HomePage } from '@/pages/home';
import { InstancePage } from '@/pages/instance';
import { SettingsPage } from '@/pages/settings';
import { ROUTE_PATTERNS } from '@/shared/config';
import { InstanceSettingsDialog } from '@/widgets/instance-settings-dialog';

import { AppRoot } from './AppRoot';

export const AppRouter: Component = () => {
  return (
    <Router root={AppRoot}>
      <Route path={ROUTE_PATTERNS.HOME} component={HomePage}>
        <Route />
        <Route
          path={ROUTE_PATTERNS.INSTANCE_SETTINGS}
          component={(props) => (
            <InstanceSettingsDialog instanceId={props.params.id} />
          )}
        />
      </Route>
      <Route path={ROUTE_PATTERNS.CONTENT} component={ContentPage} />
      <Route path={ROUTE_PATTERNS.INSTANCE} component={InstancePage}>
        <Route />
        <Route
          path={ROUTE_PATTERNS.INSTANCE_SETTINGS}
          component={(props) => (
            <InstanceSettingsDialog instanceId={props.params.id} />
          )}
        />
      </Route>
      <Route path={ROUTE_PATTERNS.SETTINGS} component={SettingsPage}>
        <Route />
      </Route>
    </Router>
  );
};
