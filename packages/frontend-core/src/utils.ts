import type { ReadModel } from "./services";

export function toMap<T extends ReadModel = ReadModel>(models: T[]): Map<string, T> {
  return new Map(models.map((model) => [`${model.modelType}:${model.id}`, model]));
}
