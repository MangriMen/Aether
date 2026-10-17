export const INSTANCE_ROUTES = {
  INSTANCE: (id: string | number) => `/instances/${encodeURIComponent(id)}`,

  INSTANCE_SETTINGS: (id: string | number) =>
    `/instances/${encodeURIComponent(id)}/settings`,

  INSTANCE_DIALOG: (id: string | number) =>
    `/instance-settings/${encodeURIComponent(id)}`,
} as const;
