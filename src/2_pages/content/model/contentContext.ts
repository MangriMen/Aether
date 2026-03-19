import type { Accessor } from 'solid-js';
import type { Store } from 'solid-js/store';

import { createContext, useContext } from 'solid-js';

import type { ContentItem, Instance } from '@/entities/instances';

import type { InstalledContentIndexStore } from './installedContentIndexStore';

export type ContentContextValue<
  T extends InstalledContentIndexStore = InstalledContentIndexStore,
> = {
  installedContentIndex: Store<T>;
  installingContentIds: Record<string, boolean>;
  providerId?: string;
  instanceId?: string;
};

export type ContentContextActions = {
  installContent: (
    item: ContentItem,
    instanceId?: Instance['id'],
  ) => Promise<void>;
  getInstancesForContent: (contentId: string) => string[];
  createIsInstalled: (
    contentId: Accessor<string | undefined>,
    instanceId: Accessor<string | undefined>,
  ) => Accessor<boolean>;
  createIsInstalling: (
    contentId: Accessor<string | undefined>,
  ) => Accessor<boolean>;
};

export type ContentContextType = [ContentContextValue, ContentContextActions];

export const ContentContext = createContext<ContentContextType>();

export const useContentContext = () => {
  const value = useContext(ContentContext);

  if (!value) {
    throw new Error(
      'useContentContext must be used within a ContentContextProvider',
    );
  }

  return value;
};
