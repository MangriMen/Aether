import { Route, Router } from '@solidjs/router';
import { Component } from 'solid-js';

import { AppLayout } from '@/widgets/layouts/app-layout';
import { MainLayout } from '@/widgets/layouts/main-layout';

import { HomePage } from '@/pages/home-page';
import { SettingsPage } from '@/pages/settings-page';

import { initializeApp } from './lib';
import { AppRoot } from './ui';

export const AppRouter: Component = () => {
  return (
    <Router root={AppRoot} rootPreload={initializeApp}>
      <Route path='*' component={AppLayout}>
        <Route path='/' component={MainLayout}>
          <Route path='/' component={HomePage} />
          <Route path='/settings' component={SettingsPage} />
        </Route>
      </Route>
    </Router>
  );
};
