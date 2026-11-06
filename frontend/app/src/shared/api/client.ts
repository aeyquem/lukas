export interface ApiFieldError {
  field: string
  code: string
}

export interface ApiErrorBody {
  error: {
    code: string
    message: string
    fields: ApiFieldError[]
  }
  request_id: string
}

export type ApiErrorKind =
  | 'bad_request'
  | 'validation'
  | 'not_found'
  | 'conflict'
  | 'unavailable'
  | 'server'
  | 'network'
  | 'unexpected'

const statusKinds: Record<number, ApiErrorKind> = {
  400: 'bad_request',
  404: 'not_found',
  409: 'conflict',
  422: 'validation',
  503: 'unavailable',
}

export class ApiClientError extends Error {
  readonly kind: ApiErrorKind
  readonly status: number
  readonly code: string
  readonly fields: ApiFieldError[]
  readonly requestId?: string
  readonly outcomeUnknown: boolean
  readonly pendingCommandKey?: string

  constructor(options: {
    kind: ApiErrorKind
    status: number
    code: string
    message: string
    fields?: ApiFieldError[]
    requestId?: string
    outcomeUnknown?: boolean
    pendingCommandKey?: string
  }) {
    super(options.message)
    this.name = 'ApiClientError'
    this.kind = options.kind
    this.status = options.status
    this.code = options.code
    this.fields = options.fields ?? []
    this.requestId = options.requestId
    this.outcomeUnknown = options.outcomeUnknown ?? false
    this.pendingCommandKey = options.pendingCommandKey
  }
}

export interface ApiRequestOptions extends Omit<RequestInit, 'body'> {
  body?: unknown
  idempotencyKey?: string
  installationGeneration?: string
}

function apiUrl(path: string): string {
  const configuredBase = window.LUKAS_RUNTIME_CONFIG?.apiBaseUrl || '/api'
  const base = configuredBase.replace(/\/+$/, '')
  const endpoint = path.replace(/^\/+/, '').replace(/^api\//, '')
  return `${base}/${endpoint}`
}

function isApiErrorBody(value: unknown): value is ApiErrorBody {
  if (typeof value !== 'object' || value === null) return false
  const body = value as Partial<ApiErrorBody>
  return (
    typeof body.request_id === 'string' &&
    typeof body.error?.code === 'string' &&
    typeof body.error.message === 'string' &&
    Array.isArray(body.error.fields) &&
    body.error.fields.every(
      (field) =>
        typeof field === 'object' &&
        field !== null &&
        typeof field.field === 'string' &&
        typeof field.code === 'string',
    )
  )
}

function errorKind(status: number): ApiErrorKind {
  return statusKinds[status] ?? (status >= 500 ? 'server' : 'unexpected')
}

function errorMessage(status: number): string {
  if (status === 503) return 'El servicio no está disponible temporalmente.'
  if (status >= 500) return 'No fue posible completar la solicitud.'
  return 'La respuesta del servicio no tiene un formato reconocido.'
}

export async function apiRequest<T>(path: string, options: ApiRequestOptions = {}): Promise<T> {
  const {
    body,
    idempotencyKey,
    installationGeneration,
    headers: suppliedHeaders,
    ...requestOptions
  } = options
  const headers = new Headers(suppliedHeaders)
  headers.set('Accept', 'application/json')

  let requestBody: BodyInit | undefined
  if (body !== undefined) {
    headers.set('Content-Type', 'application/json')
    requestBody = typeof body === 'string' ? body : JSON.stringify(body)
  }
  if (idempotencyKey) headers.set('Idempotency-Key', idempotencyKey)
  if (installationGeneration) headers.set('Installation-Generation', installationGeneration)

  const method = (requestOptions.method ?? 'GET').toUpperCase()
  const isWrite = method !== 'GET' && method !== 'HEAD' && method !== 'OPTIONS'
  let response: Response
  try {
    response = await fetch(apiUrl(path), {
      ...requestOptions,
      method,
      headers,
      body: requestBody,
    })
  } catch {
    throw new ApiClientError({
      kind: 'network',
      status: 0,
      code: 'network_error',
      message: 'No se pudo conectar con Lukas. Comprueba la conexión e inténtalo de nuevo.',
      outcomeUnknown: isWrite,
    })
  }

  if (response.status === 204) return undefined as T

  let payload: unknown
  try {
    payload = await response.json()
  } catch {
    throw new ApiClientError({
      kind: response.ok ? 'unexpected' : errorKind(response.status),
      status: response.status,
      code: 'invalid_response',
      message: errorMessage(response.status),
      outcomeUnknown: isWrite && (response.ok || response.status >= 500),
    })
  }

  if (!response.ok) {
    if (isApiErrorBody(payload)) {
      throw new ApiClientError({
        kind: errorKind(response.status),
        status: response.status,
        code: payload.error.code,
        message: payload.error.message,
        fields: payload.error.fields,
        requestId: payload.request_id,
        outcomeUnknown: isWrite && response.status >= 500,
      })
    }
    throw new ApiClientError({
      kind: errorKind(response.status),
      status: response.status,
      code: `http_${response.status}`,
      message: errorMessage(response.status),
      outcomeUnknown: isWrite && response.status >= 500,
    })
  }

  return payload as T
}
