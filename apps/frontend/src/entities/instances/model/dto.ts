import type { EditInstanceDto, NewInstanceDto } from '../api';

export type NewInstance = NewInstanceDto;

export type EditInstance = EditInstanceDto;

export type EditInstanceSettings = Omit<EditInstance, 'name'>;

export const isEditInstanceSettingsEmpty = (dto: EditInstanceSettings) =>
  Object.values(dto).every((value) => value === undefined);

export const isEditInstanceEmpty = (dto: EditInstance) =>
  isEditInstanceSettingsEmpty(dto) && dto.name === undefined;
