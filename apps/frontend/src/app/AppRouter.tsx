import { Route, Router, type RouteSectionProps } from '@solidjs/router';
import { type Component } from 'solid-js';

import { HomePage } from '@/app/pages/home';
import { ContentBrowserWrapper } from '@/features/content-browser';
import { ContentItemPage } from '@/features/content-item';
import { ContentPageLayout } from '@/features/content-page';
import { InstanceSettingsDialog } from '@/features/instance-edit';
import { InstancePage } from '@/features/instance-page';

import { AppRoot } from './AppRoot';
import { ROUTE_PATTERNS } from './routePatterns';

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
    </Router>
  );
};
