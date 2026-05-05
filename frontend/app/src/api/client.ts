import type { ApiError } from "./errors";
import type { Result } from "./result";

const defaultBaseUrl = "http://127.0.0.1:5000";

export function getApiBaseUrl(): string {
  return import.meta.env.VITE_LUKAS_API_BASE_URL ?? defaultBaseUrl;
}

export async function getJson<T>(path: string): Promise<Result<T, ApiError>> {
  const response = await fetch(`${getApiBaseUrl()}${path}`);

  if (!response.ok) {
    return {
      ok: false,
      error: {
        status: response.status,
        title: response.statusText,
      },
    };
  }

  return { ok: true, value: (await response.json()) as T };
}
