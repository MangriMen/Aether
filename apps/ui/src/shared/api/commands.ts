import { commands as authRawCommands } from './bindings/auth';
import { withIdempotency } from './tauri';

export const authCommands = {
  listAccounts: authRawCommands.listAccounts,
  createOfflineAccount: withIdempotency(authRawCommands.createOfflineAccount),
  changeAccount: withIdempotency(authRawCommands.changeAccount),
  logout: withIdempotency(authRawCommands.logout),
};
