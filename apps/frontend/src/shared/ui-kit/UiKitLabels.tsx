import { createContext, useContext, type JSX } from 'solid-js';

/**
 * Texts that ui-kit components render on their own (close buttons, empty
 * tables, pagination). ui-kit does not depend on i18n: the app provides them.
 */
export type UiKitLabels = {
  close: string | undefined;
  noData: string | undefined;
  morePages: string | undefined;
  previous: string | undefined;
  next: string | undefined;
  ok: string | undefined;
  cancel: string | undefined;
  browse: string | undefined;
};

const DEFAULT_UI_KIT_LABELS: UiKitLabels = {
  close: 'Close',
  noData: 'No data to display',
  morePages: 'More pages',
  previous: 'Previous',
  next: 'Next',
  ok: 'Ok',
  cancel: 'Cancel',
  browse: 'Browse',
};

const UiKitLabelsContext = createContext<UiKitLabels>(DEFAULT_UI_KIT_LABELS);

export type UiKitLabelsProviderProps = {
  /** Pass getters to keep the labels reactive (e.g. to the current locale). */
  labels: UiKitLabels;
  children?: JSX.Element;
};

export const UiKitLabelsProvider = (props: UiKitLabelsProviderProps) => (
  // eslint-disable-next-line solid/reactivity
  <UiKitLabelsContext.Provider value={props.labels}>
    {props.children}
  </UiKitLabelsContext.Provider>
);

export const useUiKitLabels = () => useContext(UiKitLabelsContext);
