import type { Accessor } from 'solid-js';
import type { Store } from 'solid-js/store';

import { createContext, useContext } from 'solid-js';

import type {
  ContentFilters,
  ContentItem,
  Instance,
  ProviderId,
} from '../../../entities/instances';
import type { ContentFiltersLock } from './contentFiltersLock';
import type { InstalledContentIndexStore } from './installedContentIndexStore';

export type ContentContextValue<
  T extends InstalledContentIndexStore = InstalledContentIndexStore,
> = {
  installedContentIndex: Store<T>;
  installingContentIds: Record<string, boolean>;
  providerId?: ProviderId;
  instanceId?: string;
  filters?: ContentFilters;
  filtersLock?: ContentFiltersLock;
};

export type ContentContextActions = {
  installContent: (
    item: Pick<ContentItem, 'id' | 'contentType'>,
    instanceId?: Instance['id'],
    contentVersion?: string,
  ) => Promise<void>;
  createIsInstalled: (
    contentId: Accessor<string | undefined>,
    instanceId: Accessor<string | undefined>,
  ) => Accessor<boolean>;
  createIsInstalling: (
    contentId: Accessor<string | undefined>,
  ) => Accessor<boolean>;
  createInstalledVersion: (
    contentId: Accessor<string | undefined>,
    instanceId: Accessor<string | undefined>,
  ) => Accessor<string | undefined>;
  setFilters?: (filters: ContentFilters) => void;
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
