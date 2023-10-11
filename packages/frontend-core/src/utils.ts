import type { Model } from "./services";

export function toMap<T extends Model = Model>(models: T[]): Map<string, T> {
  return new Map(models.map((model) => [`${model.modelType}:${model.id}`, model]));
}

export const NULL_PLACEHOLDER = "(null)" as const;
