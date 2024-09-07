import { Component } from 'solid-js';

import { HomePageLayout } from '../HomePageLayout';

import { HomePageProps } from '.';

export const HomePage: Component<HomePageProps> = (props) => {
  return (
    <HomePageLayout>
      <div {...props}>HomePage</div>
    </HomePageLayout>
  );
};
