import { Route, Router, type RouteSectionProps } from '@solidjs/router';
import { type Component, lazy } from 'solid-js';

import { HomePage } from '@/app/pages/home';

import { AppRoot } from './AppRoot';
import { ROUTE_PATTERNS } from './routePatterns';

// The pages other than the home page load when they are first opened.
const ContentBrowserWrapper = lazy(() =>
  import('@/features/content-browser').then((m) => ({
    default: m.ContentBrowserWrapper,
  })),
);
const ContentItemPage = lazy(() =>
  import('@/features/content-item').then((m) => ({
    default: m.ContentItemPage,
  })),
);
const ContentPageLayout = lazy(() =>
  import('@/features/content-page').then((m) => ({
    default: m.ContentPageLayout,
  })),
);
const InstanceSettingsDialog = lazy(() =>
  import('@/features/instance-edit').then((m) => ({
    default: m.InstanceSettingsDialog,
  })),
);
const InstancePage = lazy(() =>
  import('@/features/instance-page').then((m) => ({
    default: m.InstancePage,
  })),
);

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
        >
          <Route path='/' />
          <Route path={ROUTE_PATTERNS.CONTENT_ITEM_VERSIONS_REL} />
        </Route>
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
