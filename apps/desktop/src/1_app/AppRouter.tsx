import { Route, Router } from '@solidjs/router';
import { lazy, type Component } from 'solid-js';

import {
  ContentBrowserWrapper,
  ContentItemPage,
  ContentPageLayout,
} from '@/pages/content';
import { HomePage } from '@/pages/home';
import { InstancePage } from '@/pages/instance';
import { SettingsPage } from '@/pages/settings';
import { ROUTE_PATTERNS } from '@/shared/config';
import { InstanceSettingsDialog } from '@/widgets/instance-settings-dialog';

const PlaygroundPage = lazy(() =>
  import('@/pages/playground').then((m) => ({ default: m.PlaygroundPage })),
);

import { AppRoot } from './AppRoot';

export const AppRouter: Component = () => {
  return (
    <Router root={AppRoot}>
      <Route path={ROUTE_PATTERNS.HOME} component={HomePage}>
        <Route />
        <Route
          path={ROUTE_PATTERNS.INSTANCE_DIALOG}
          component={(props) => (
            <InstanceSettingsDialog instanceId={props.params.id} />
          )}
        />
      </Route>
      <Route path={ROUTE_PATTERNS.CONTENT} component={ContentPageLayout}>
        <Route path='/' component={ContentBrowserWrapper} />
        <Route
          path={ROUTE_PATTERNS.CONTENT_ITEM_REL}
          component={ContentItemPage}
        />
      </Route>
      <Route path={ROUTE_PATTERNS.INSTANCE} component={InstancePage}>
        <Route />
        <Route
          path={ROUTE_PATTERNS.INSTANCE_SETTINGS_REL}
          component={(props) => (
            <InstanceSettingsDialog instanceId={props.params.id} />
          )}
        />
      </Route>
      <Route path={ROUTE_PATTERNS.SETTINGS} component={SettingsPage}>
        <Route />
      </Route>
      <Route path={'/playground'} component={PlaygroundPage} />
    </Router>
  );
};
