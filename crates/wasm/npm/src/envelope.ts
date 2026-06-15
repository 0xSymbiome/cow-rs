export type SchemaVersion = "v1" | "__unknown";

export interface WasmEnvelope<T> {
  schemaVersion: SchemaVersion;
  value: T;
}
