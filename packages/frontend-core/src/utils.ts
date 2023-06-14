import type { ReadModel } from "./services";

export function toMap(models: ReadModel[]): Map<string, ReadModel> {
  return new Map(models.map((model) => [`${model.modelType}:${model.id}`, model]));
}
