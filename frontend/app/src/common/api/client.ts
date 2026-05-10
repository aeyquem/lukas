import type { ApiError } from "@/common/api/errors";
import type { Result } from "@/common/api/result";

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

export async function postJson<TResponse, TRequest>(
  path: string,
  body: TRequest,
): Promise<Result<TResponse, ApiError>> {
  const response = await fetch(`${getApiBaseUrl()}${path}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });

  if (!response.ok) {
    const problem = await readProblem(response);
    return {
      ok: false,
      error: {
        status: response.status,
        title: problem?.title ?? response.statusText,
        detail: problem?.detail,
      },
    };
  }

  return { ok: true, value: (await response.json()) as TResponse };
}

export async function putJson<TResponse, TRequest>(
  path: string,
  body: TRequest,
): Promise<Result<TResponse, ApiError>> {
  const response = await fetch(`${getApiBaseUrl()}${path}`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });

  if (!response.ok) {
    const problem = await readProblem(response);
    return {
      ok: false,
      error: {
        status: response.status,
        title: problem?.title ?? response.statusText,
        detail: problem?.detail,
      },
    };
  }

  return { ok: true, value: (await response.json()) as TResponse };
}

async function readProblem(
  response: Response,
): Promise<{ title?: string; detail?: string } | null> {
  const contentType = response.headers.get("Content-Type");
  if (!contentType?.includes("application/problem+json")) {
    return null;
  }

  return (await response.json()) as { title?: string; detail?: string };
}
