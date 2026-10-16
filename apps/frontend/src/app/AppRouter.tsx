import { Route, Router, type RouteSectionProps } from '@solidjs/router';
import { lazy, type Component } from 'solid-js';

import { InstanceSettingsDialog } from '@/features/instance-edit';
import { InstancePage } from '@/features/instance-page';
import {
  ContentBrowserWrapper,
  ContentItemPage,
  ContentPageLayout,
} from '@/pages/content';
import { HomePage } from '@/pages/home';
import { ROUTE_PATTERNS } from '@/shared/config';

const PlaygroundPage = lazy(() =>
  import('@/pages/playground').then((m) => ({ default: m.PlaygroundPage })),
);

import { AppRoot } from './AppRoot';

const InstanceSettingsRoute: Component<RouteSectionProps> = (props) => (
  <InstanceSettingsDialog instanceId={props.params.id} />
);

export const AppRouter: Component = () => {
  return (
    <Router root={AppRoot}>
      <Route path={ROUTE_PATTERNS.HOME} component={HomePage}>
        <Route />
        <Route
          path={ROUTE_PATTERNS.INSTANCE_DIALOG}
          component={InstanceSettingsRoute}
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
          component={InstanceSettingsRoute}
        />
      </Route>

      <Route path={ROUTE_PATTERNS.PLAYGROUND} component={PlaygroundPage} />
    </Router>
  );
};
