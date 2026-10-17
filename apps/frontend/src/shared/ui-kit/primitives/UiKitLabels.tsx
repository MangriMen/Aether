import { createContext, mergeProps, useContext, type JSX } from 'solid-js';

/**
 * Texts that ui-kit components render on their own (close buttons, empty
 * tables, pagination). ui-kit does not depend on i18n: the app provides them.
 */
export type UiKitLabels = {
  close: string;
  noData: string;
  morePages: string;
  previous: string;
  next: string;
  ok: string;
  cancel: string;
  browse: string;
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
  /** A missing (undefined) label falls back to the English default. */
  labels: { [K in keyof UiKitLabels]?: string | undefined };
  children?: JSX.Element;
};

export const UiKitLabelsProvider = (props: UiKitLabelsProviderProps) => {
  // mergeProps reads its sources on access, so the labels follow the locale
  // whether they come as getters or as a new object.
  const value = mergeProps(
    DEFAULT_UI_KIT_LABELS,
    () => props.labels,
  ) as UiKitLabels;

  return (
    <UiKitLabelsContext.Provider value={value}>
      {props.children}
    </UiKitLabelsContext.Provider>
  );
};

export const useUiKitLabels = () => useContext(UiKitLabelsContext);
