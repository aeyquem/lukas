import { ApiClientError, apiRequest } from './client'

export type FinancialWriteMethod = 'POST' | 'PUT' | 'DELETE'

export interface PendingFinancialWrite {
  method: FinancialWriteMethod
  route: string
  body: string
  idempotencyKey: string
  installationGeneration: string
  createdAt: string
}

const storagePrefix = 'lukas.pending-financial-write.v1:'
const uuidPattern = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i

export class PendingWriteStorageError extends Error {
  constructor(cause?: unknown) {
    super('No se pudo guardar el comando pendiente. No se envió la operación.', { cause })
    this.name = 'PendingWriteStorageError'
  }
}

export class InstallationGenerationMismatchError extends Error {
  constructor() {
    super('Esta instalación cambió. El comando pendiente se conserva y no se reenvió.')
    this.name = 'InstallationGenerationMismatchError'
  }
}

export class PendingWriteExistsError extends Error {
  constructor() {
    super('La clave ya tiene un comando pendiente. Usa Retry para reenviar el comando original.')
    this.name = 'PendingWriteExistsError'
  }
}

/**
 * Storage wrapper that fails closed. Corrupt data is never silently discarded,
 * because it may be the only copy of a request whose outcome is unknown.
 */
export class PendingWriteStore {
  constructor(private readonly storage: Storage = browserStorage()) {}

  list(): PendingFinancialWrite[] {
    const commands: PendingFinancialWrite[] = []
    try {
      for (let index = 0; index < this.storage.length; index += 1) {
        const key = this.storage.key(index)
        if (!key?.startsWith(storagePrefix)) continue
        const raw = this.storage.getItem(key)
        // Another tab may have completed and removed the command since key()
        // returned it. A missing value is therefore a harmless race.
        if (raw === null) continue
        const command: unknown = JSON.parse(raw)
        if (!isPendingWrite(command)) throw new TypeError('Invalid pending command.')
        commands.push({ ...command })
      }
    } catch (cause) {
      if (cause instanceof PendingWriteStorageError) throw cause
      throw new PendingWriteStorageError(cause)
    }
    return commands.sort((left, right) => left.createdAt.localeCompare(right.createdAt))
  }

  get(idempotencyKey: string): PendingFinancialWrite | undefined {
    let raw: string | null
    try {
      raw = this.storage.getItem(storageKey(idempotencyKey))
    } catch (cause) {
      throw new PendingWriteStorageError(cause)
    }
    if (raw === null) return undefined
    try {
      const command: unknown = JSON.parse(raw)
      if (!isPendingWrite(command) || command.idempotencyKey !== idempotencyKey) {
        throw new TypeError('Invalid pending command.')
      }
      return command
    } catch (cause) {
      throw new PendingWriteStorageError(cause)
    }
  }

  saveNew(command: PendingFinancialWrite): void {
    if (this.get(command.idempotencyKey)) {
      throw new PendingWriteExistsError()
    }
    try {
      this.storage.setItem(storageKey(command.idempotencyKey), JSON.stringify(command))
    } catch (cause) {
      throw new PendingWriteStorageError(cause)
    }
  }

  remove(idempotencyKey: string): void {
    try {
      this.storage.removeItem(storageKey(idempotencyKey))
    } catch (cause) {
      throw new PendingWriteStorageError(cause)
    }
  }

  /** Dismissing the UI notice deliberately leaves the command in origin storage. */
  dismiss(idempotencyKey: string): PendingFinancialWrite | undefined {
    return this.get(idempotencyKey)
  }

}

function browserStorage(): Storage {
  try {
    return window.localStorage
  } catch (cause) {
    throw new PendingWriteStorageError(cause)
  }
}

function storageKey(idempotencyKey: string): string {
  if (!uuidPattern.test(idempotencyKey)) throw new TypeError('Idempotency key must be a UUID.')
  return `${storagePrefix}${idempotencyKey}`
}

function isPendingWrite(value: unknown): value is PendingFinancialWrite {
  if (typeof value !== 'object' || value === null) return false
  const command = value as Partial<PendingFinancialWrite>
  return (
    (command.method === 'POST' || command.method === 'PUT' || command.method === 'DELETE') &&
    typeof command.route === 'string' &&
    command.route.startsWith('/') &&
    typeof command.body === 'string' &&
    typeof command.idempotencyKey === 'string' &&
    uuidPattern.test(command.idempotencyKey) &&
    typeof command.installationGeneration === 'string' &&
    uuidPattern.test(command.installationGeneration) &&
    typeof command.createdAt === 'string'
  )
}

function newIdempotencyKey(): string {
  const key = globalThis.crypto?.randomUUID?.()
  if (!key) throw new Error('Secure UUID generation is unavailable.')
  return key
}

function isUnknownOutcome(error: unknown): boolean {
  return !(error instanceof ApiClientError) || error.outcomeUnknown
}

function attachPendingKey(error: unknown, idempotencyKey: string): Error {
  if (error instanceof ApiClientError) {
    return new ApiClientError({
      kind: error.kind,
      status: error.status,
      code: error.code,
      message: error.message,
      fields: error.fields,
      requestId: error.requestId,
      outcomeUnknown: true,
      pendingCommandKey: idempotencyKey,
    })
  }
  const wrapped = new Error('El resultado de la operación es desconocido. Conservamos el comando para reintentar.', {
    cause: error,
  })
  wrapped.name = 'PendingWriteOutcomeUnknownError'
  Object.assign(wrapped, { pendingCommandKey: idempotencyKey })
  return wrapped
}

async function sendStoredCommand<T>(
  command: PendingFinancialWrite,
  store: PendingWriteStore,
): Promise<T> {
  try {
    const result = await apiRequest<T>(command.route, {
      method: command.method,
      body: command.body,
      idempotencyKey: command.idempotencyKey,
      installationGeneration: command.installationGeneration,
    })
    // If removal fails after a successful response, an exact retry remains safe
    // because the server receipt is keyed by this same method, route, and body.
    try {
      store.remove(command.idempotencyKey)
    } catch {
      // Keep the successful result; a later retry can recover its saved receipt.
    }
    return result
  } catch (error) {
    if (isUnknownOutcome(error)) throw attachPendingKey(error, command.idempotencyKey)
    try {
      store.remove(command.idempotencyKey)
    } catch {
      // A stale exact command is safe to retry; never mask the definite response.
    }
    throw error
  }
}

export interface FinancialWriteIntent {
  method: FinancialWriteMethod
  route: string
  /** Serialize the request once and pass those exact bytes on every retry. */
  body: string
  idempotencyKey?: string
}

/**
 * Fetch the server-issued installation/database generation before allowing a
 * financial write. Bind it to the exact pending command for retry protection.
 */
export async function fetchInstallationGeneration(): Promise<string> {
  const response = await apiRequest<{ installation_generation: string }>('/bootstrap', {
    cache: 'no-store',
  })
  if (!uuidPattern.test(response.installation_generation)) {
    throw new Error('The server returned an invalid installation generation.')
  }
  return response.installation_generation
}

/** Persist the exact command before sending it to the backend. */
export async function sendFinancialWrite<T>(
  intent: FinancialWriteIntent,
  store = new PendingWriteStore(),
): Promise<T> {
  const installationGeneration = await fetchInstallationGeneration()
  const idempotencyKey = intent.idempotencyKey ?? newIdempotencyKey()
  const command: PendingFinancialWrite = {
    method: intent.method,
    route: intent.route,
    body: intent.body,
    idempotencyKey,
    installationGeneration,
    createdAt: new Date().toISOString(),
  }
  if (!isPendingWrite(command)) throw new TypeError('Financial write command is invalid.')

  // This write must succeed before `fetch` is reached.
  store.saveNew(command)
  return sendStoredCommand<T>(command, store)
}

/** Resend the stored bytes, method, route, and key after an explicit Retry. */
export async function retryPendingWrite<T>(
  idempotencyKey: string,
  store = new PendingWriteStore(),
): Promise<T> {
  const command = store.get(idempotencyKey)
  if (!command) throw new Error('The pending command no longer exists.')
  const currentInstallationGeneration = await fetchInstallationGeneration()
  if (command.installationGeneration !== currentInstallationGeneration) {
    throw new InstallationGenerationMismatchError()
  }
  return sendStoredCommand<T>(command, store)
}

/** Hiding a notice never deletes the unresolved command. */
export function dismissPendingWrite(
  idempotencyKey: string,
  store = new PendingWriteStore(),
): PendingFinancialWrite | undefined {
  return store.dismiss(idempotencyKey)
}
