/** Installing modpacks is switched off by a feature flag; nothing failed, the action is just not allowed. */
export class ModpackInstallDisabledError extends Error {
  constructor() {
    super('Installing modpacks is disabled');
    this.name = 'ModpackInstallDisabledError';
  }
}
