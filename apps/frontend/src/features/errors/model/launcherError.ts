import type { FrontendError } from '@/platform/bindings/application';
import type { TFunction } from '@/shared/i18n';

import { dayjs, formatDuration } from '@/shared/format';

export type LauncherError = FrontendError;

export function isLauncherError(error: unknown): error is FrontendError {
  return (
    error !== null &&
    typeof error === 'object' &&
    'type' in error &&
    'payload' in error &&
    typeof error.type === 'string'
  );
}

/** The user called the install off: nothing failed, so nothing to report. */
export const isCancelledError = (error: unknown): boolean =>
  isLauncherError(error) &&
  (error.type === 'instance' || error.type === 'java') &&
  error.payload.code === 'CANCELLED';

/** Nothing categorized the failure, so its raw text is the only thing there is to show. */
export const isInternalError = (error: unknown): boolean =>
  isLauncherError(error) &&
  error.type === 'application' &&
  error.payload.code === 'INTERNAL';

/** The provider does not do what was asked of it (an optional operation it lacks). */
export const isUnsupportedOperationError = (error: unknown): boolean =>
  isLauncherError(error) &&
  error.type === 'instance' &&
  error.payload.code === 'UNSUPPORTED_OPERATION';

type LauncherErrorTranslateKey<E extends LauncherError> =
  E extends LauncherError ? `${E['type']}.${E['payload']['code']}` : never;

type LauncherErrorArgs<E extends LauncherError> = E extends LauncherError
  ? E['payload'] extends { payload: infer P }
    ? P
    : Record<string, never>
  : never;

const getLauncherErrorTranslationData = <E extends LauncherError>(
  error: E,
): {
  key: LauncherErrorTranslateKey<E>;
  args: LauncherErrorArgs<E> | undefined;
} => {
  const key = `${error.type}.${error.payload.code}`;
  const args = 'payload' in error.payload ? error.payload.payload : undefined;

  return {
    key: key as LauncherErrorTranslateKey<E>,
    args: args as LauncherErrorArgs<E>,
  };
};

const isRecord = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === 'object';

const getIoCauseText = (kind: unknown, t: TFunction): string | undefined => {
  switch (kind) {
    case 'notFound':
      return t('errors.cause.io.notFound');
    case 'permissionDenied':
      return t('errors.cause.io.permissionDenied');
    case 'alreadyExists':
      return t('errors.cause.io.alreadyExists');
    case 'storageFull':
      return t('errors.cause.io.storageFull');
    case 'readOnlyFilesystem':
      return t('errors.cause.io.readOnly');
    case 'resourceBusy':
      return t('errors.cause.io.busy');
    case 'timedOut':
      return t('errors.cause.io.timedOut');
    case 'invalidData':
      return t('errors.cause.io.invalidData');
    default:
      return undefined;
  }
};

const getNetworkCauseText = (
  kind: unknown,
  t: TFunction,
): string | undefined => {
  switch (kind) {
    case 'timeout':
      return t('errors.cause.network.timeout');
    case 'connect':
      return t('errors.cause.network.connect');
    case 'redirect':
      return t('errors.cause.network.redirect');
    case 'body':
      return t('errors.cause.network.body');
    default:
      return undefined;
  }
};

/** The translation of the category of a failure, or undefined when it has none to name. */
const getCauseText = (cause: unknown, t: TFunction): string | undefined => {
  if (!isRecord(cause)) {
    return undefined;
  }

  switch (cause.type) {
    case 'io':
      return getIoCauseText(cause.kind, t);
    case 'network':
      return getNetworkCauseText(cause.kind, t);
    case 'httpStatus':
      return t('errors.cause.httpStatus', { status: Number(cause.status) });
    case 'parse':
      return typeof cause.line === 'number' && typeof cause.column === 'number'
        ? t('errors.cause.parse', { line: cause.line, column: cause.column })
        : t('errors.cause.parseNoPosition');
    case 'database':
      return t('errors.cause.database');
    default:
      return undefined;
  }
};

/** A translatable reason (`{ code, payload? }`) replaces the raw argument of the same name. */
const withTranslatedReason = (
  key: string,
  args: unknown,
  t: TFunction,
): unknown => {
  if (!isRecord(args) || !isRecord(args.reason)) {
    return args;
  }

  const { code, payload } = args.reason;

  if (typeof code !== 'string') {
    return args;
  }

  const reasonKey = `errors.${key.replace('.', '.reason.')}.${code}`;

  return {
    ...args,
    reason:
      t(reasonKey as Parameters<TFunction>[0], payload as never) ?? reasonKey,
  };
};

/** An argument named `field` is a raw name of a field, translated by `errors.<type>.field.<NAME>` when there is one. */
const withTranslatedField = (
  key: string,
  args: unknown,
  t: TFunction,
): unknown => {
  if (!isRecord(args) || typeof args.field !== 'string') {
    return args;
  }

  const fieldKey = `errors.${key.split('.')[0]}.field.${args.field}`;
  const translated = t(fieldKey as Parameters<TFunction>[0]);

  return typeof translated === 'string' ? { ...args, field: translated } : args;
};

/** The arguments of an error: its own payload, or that of its reason when the cause is the reason's. */
const getCauseOf = (inner: unknown): unknown => {
  if (!isRecord(inner)) {
    return undefined;
  }

  if (inner.cause !== undefined) {
    return inner.cause;
  }

  return isRecord(inner.reason) && isRecord(inner.reason.payload)
    ? inner.reason.payload.cause
    : undefined;
};

const getCause = (error: LauncherError): unknown => {
  const payload: unknown = error.payload;

  return isRecord(payload) ? getCauseOf(payload.payload) : undefined;
};

export const getTranslatedError = (
  error: LauncherError,
  t: TFunction,
): string => {
  const { key, args } = getLauncherErrorTranslationData(error);

  const main =
    t(
      `errors.${key}`,
      withFormattedArgs(
        key,
        withTranslatedReason(key, withTranslatedField(key, args, t), t),
      ),
    ) ?? key;
  const causeText = getCauseText(getCause(error), t);

  return causeText ? `${main}: ${causeText}` : main;
};

/**
 * The raw text of a failure nobody could categorize, looked up in the error and in the
 * errors nested in it. Only `ErrorDetails` may show it.
 */
export const getErrorDetails = (error: unknown): string | undefined => {
  if (!isLauncherError(error)) {
    return undefined;
  }

  const payload: unknown = error.payload;
  const inner = isRecord(payload) ? payload.payload : undefined;

  if (!isRecord(inner)) {
    return undefined;
  }

  const cause = getCauseOf(inner);

  if (isRecord(cause) && cause.type === 'unknown') {
    return typeof cause.details === 'string' ? cause.details : undefined;
  }

  return getErrorDetails(inner.error);
};

/** Values the core sends as raw numbers, turned into text in the language of the interface. */
const withFormattedArgs = (
  key: string,
  args: unknown,
): Parameters<TFunction>[1] => {
  if (
    key === 'instance.PRELAUNCH_COMMAND_TIMEOUT' &&
    args !== null &&
    typeof args === 'object' &&
    'seconds' in args
  ) {
    return {
      duration: formatDuration(Number(args.seconds), dayjs.locale()),
    };
  }

  return args as Parameters<TFunction>[1];
};
