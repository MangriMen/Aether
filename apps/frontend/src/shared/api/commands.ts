import { commands as applicationRawCommands } from './bindings/application';
import { commands as authRawCommands } from './bindings/auth';
import {
  commands as eventsRawCommands,
  events as eventsRawEvents,
} from './bindings/events';
import {
  commands as instanceRawCommands,
  events as instanceRawEvents,
} from './bindings/instance';
import { commands as javaRawCommands } from './bindings/java';
import { commands as minecraftRawCommands } from './bindings/minecraft';
import {
  commands as pluginsRawCommands,
  events as pluginsRawEvents,
} from './bindings/plugin';
import {
  commands as processRawCommands,
  events as processRawEvents,
} from './bindings/process';
import { commands as settingsRawCommands } from './bindings/settings';
import { commands as updateRawCommands } from './bindings/update';
import { withIdempotency } from './tauri';

export const authCommands = {
  listAccounts: authRawCommands.listAccounts,
  createOfflineAccount: withIdempotency(authRawCommands.createOfflineAccount),
  changeAccount: withIdempotency(authRawCommands.changeAccount),
  logout: withIdempotency(authRawCommands.logout),
};

export const updateCommands = {
  checkForUpdates: updateRawCommands.checkForUpdates,
  installUpdate: withIdempotency(updateRawCommands.installUpdate),
};

export const pluginsCommands = {
  import: withIdempotency(pluginsRawCommands.import),
  sync: withIdempotency(pluginsRawCommands.sync),
  list: pluginsRawCommands.list,
  get: pluginsRawCommands.get,
  remove: withIdempotency(pluginsRawCommands.remove),
  enable: withIdempotency(pluginsRawCommands.enable),
  disable: withIdempotency(pluginsRawCommands.disable),
  call: withIdempotency(pluginsRawCommands.call),
  getSettings: pluginsRawCommands.getSettings,
  editSettings: withIdempotency(pluginsRawCommands.editSettings),
  openPluginsFolder: pluginsRawCommands.openPluginsFolder,
  getApiVersion: pluginsRawCommands.getApiVersion,
};

export const settingsCommands = {
  get: settingsRawCommands.get,
  edit: withIdempotency(settingsRawCommands.edit),
  getMaxRam: settingsRawCommands.getMaxRam,
  getDefaultInstanceSettings: settingsRawCommands.getDefaultInstanceSettings,
  editDefaultInstanceSettings: withIdempotency(
    settingsRawCommands.editDefaultInstanceSettings,
  ),
  getAppSettings: settingsRawCommands.getAppSettings,
  editAppSettings: withIdempotency(settingsRawCommands.editAppSettings),
};

export const instanceCommands = {
  create: withIdempotency(instanceRawCommands.create),
  import: withIdempotency(instanceRawCommands.import),
  listImporters: instanceRawCommands.listImporters,
  list: instanceRawCommands.list,
  get: instanceRawCommands.get,
  getDir: instanceRawCommands.getDir,
  install: withIdempotency(instanceRawCommands.install),
  update: withIdempotency(instanceRawCommands.update),
  edit: withIdempotency(instanceRawCommands.edit),
  remove: withIdempotency(instanceRawCommands.remove),
  launch: withIdempotency(instanceRawCommands.launch),
  stop: withIdempotency(instanceRawCommands.stop),
  importContents: withIdempotency(instanceRawCommands.importContents),
  listContent: instanceRawCommands.listContent,
  installContent: withIdempotency(instanceRawCommands.installContent),
  enableContents: withIdempotency(instanceRawCommands.enableContents),
  disableContents: withIdempotency(instanceRawCommands.disableContents),
  removeContents: withIdempotency(instanceRawCommands.removeContents),
  listContentProviders: instanceRawCommands.listContentProviders,
  searchContent: instanceRawCommands.searchContent,
  checkCompatibility: instanceRawCommands.checkCompatibility,
  getContent: instanceRawCommands.getContent,
  listContentVersion: instanceRawCommands.listContentVersion,
  editIcon: withIdempotency(instanceRawCommands.editIcon),
};

export const javaCommands = {
  list: javaRawCommands.list,
  edit: withIdempotency(javaRawCommands.edit),
  remove: withIdempotency(javaRawCommands.remove),
  install: withIdempotency(javaRawCommands.install),
  testJre: javaRawCommands.testJre,
  discover: javaRawCommands.discover,
  getActiveInstallations: javaRawCommands.getActiveInstallations,
};

export const minecraftCommands = {
  getMinecraftVersionManifest: minecraftRawCommands.getMinecraftVersionManifest,
  getLoaderVersionManifest: minecraftRawCommands.getLoaderVersionManifest,
};

export const processCommands = {
  list: processRawCommands.list,
  getByInstanceId: processRawCommands.getByInstanceId,
};

export const eventsCommands = {
  listProgressBars: eventsRawCommands.listProgressBars,
};

export const applicationCommands = {
  waitForInitialization: applicationRawCommands.waitForInitialization,
  initializePlugins: applicationRawCommands.initializePlugins,
  recreateWindow: applicationRawCommands.recreateWindow,
};

// Re-export events from bindings
export const pluginsEvents = pluginsRawEvents;
export const instanceEvents = instanceRawEvents;
export const processEvents = processRawEvents;
export const eventsEvents = eventsRawEvents;
