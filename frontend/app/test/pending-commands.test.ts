import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  ApiClientError,
} from '../src/shared/api/client'
import {
  InstallationGenerationMismatchError,
  PendingWriteStorageError,
  PendingWriteStore,
  dismissPendingWrite,
  retryPendingWrite,
  sendFinancialWrite,
  type FinancialWriteIntent,
} from '../src/shared/api/pending-commands'

const key = '237a2988-2a9e-4d49-830b-1244d5a1f01a'
const generation = 'd48ecc15-a9ce-4a82-8339-53e0697a9c8f'
const newerGeneration = 'd42f0933-8af2-41f1-96c5-e21459fbb0a2'
const exactBody = '{ "name":"Everyday", "amount":"100.20" }'

class MemoryStorage implements Storage {
  private readonly values = new Map<string, string>()

  get length(): number {
    return this.values.size
  }

  clear(): void {
    this.values.clear()
  }

  getItem(storageKey: string): string | null {
    return this.values.get(storageKey) ?? null
  }

  key(index: number): string | null {
    return [...this.values.keys()][index] ?? null
  }

  removeItem(storageKey: string): void {
    this.values.delete(storageKey)
  }

  setItem(storageKey: string, value: string): void {
    this.values.set(storageKey, value)
  }
}

class UnavailableStorage extends MemoryStorage {
  override setItem(): void {
    throw new Error('Storage is disabled')
  }
}

function intent(): FinancialWriteIntent {
  return {
    method: 'POST',
    route: '/api/accounts',
    body: exactBody,
    idempotencyKey: key,
  }
}

function bootstrapResponse(installationGeneration = generation): Response {
  return new Response(JSON.stringify({ installation_generation: installationGeneration }), {
    status: 200,
    headers: { 'content-type': 'application/json' },
  })
}

function successResponse(): Response {
  return new Response(JSON.stringify({ resource_id: key, revision: 1 }), {
    status: 201,
    headers: { 'content-type': 'application/json' },
  })
}

let fetchMock: ReturnType<typeof vi.fn>

beforeEach(() => {
  fetchMock = vi.fn()
  vi.stubGlobal('fetch', fetchMock)
  vi.stubGlobal('window', { LUKAS_RUNTIME_CONFIG: { apiBaseUrl: '/api' } })
})

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('durable financial write commands', () => {
  it('does not send a write if storage fails before persisting the exact body', async () => {
    const store = new PendingWriteStore(new UnavailableStorage())
    fetchMock.mockResolvedValueOnce(bootstrapResponse())

    await expect(sendFinancialWrite(intent(), store)).rejects.toBeInstanceOf(
      PendingWriteStorageError,
    )
    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0]?.[0]).toBe('/api/bootstrap')
    expect((fetchMock.mock.calls[0]?.[1] as RequestInit).cache).toBe('no-store')
  })

  it('keeps an unknown command across dismissal and reload, then retries exact bytes', async () => {
    const storage = new MemoryStorage()
    const firstStore = new PendingWriteStore(storage)
    fetchMock.mockResolvedValueOnce(bootstrapResponse())
    fetchMock.mockRejectedValueOnce(new TypeError('connection dropped'))

    const error = await sendFinancialWrite(intent(), firstStore).catch((caught: unknown) => caught)
    expect(error).toBeInstanceOf(ApiClientError)
    expect(error).toMatchObject({ pendingCommandKey: key, outcomeUnknown: true })
    expect(dismissPendingWrite(key, firstStore)?.body).toBe(exactBody)

    const reloadedStore = new PendingWriteStore(storage)
    expect(reloadedStore.list()).toHaveLength(1)
    fetchMock.mockResolvedValueOnce(bootstrapResponse())
    fetchMock.mockResolvedValueOnce(successResponse())
    await expect(retryPendingWrite(key, reloadedStore)).resolves.toEqual({
      resource_id: key,
      revision: 1,
    })

    const [, retryOptions] = fetchMock.mock.calls[3] as [string, RequestInit]
    expect(fetchMock.mock.calls[3]?.[0]).toBe('/api/accounts')
    expect(retryOptions.method).toBe('POST')
    expect(retryOptions.body).toBe(exactBody)
    expect(new Headers(retryOptions.headers).get('Idempotency-Key')).toBe(key)
    expect(new Headers(retryOptions.headers).get('Installation-Generation')).toBe(generation)
    const [, initialOptions] = fetchMock.mock.calls[1] as [string, RequestInit]
    expect(new Headers(initialOptions.headers).get('Installation-Generation')).toBe(generation)
    expect(reloadedStore.list()).toHaveLength(0)
  })

  it('keeps 503 writes unresolved and leaves them persisted when the notice is dismissed', async () => {
    const storage = new MemoryStorage()
    const store = new PendingWriteStore(storage)
    fetchMock.mockResolvedValueOnce(bootstrapResponse())
    fetchMock.mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          error: { code: 'storage_unavailable', message: 'Unavailable', fields: [] },
          request_id: key,
        }),
        { status: 503, headers: { 'content-type': 'application/json' } },
      ),
    )

    const error = await sendFinancialWrite(intent(), store).catch((caught: unknown) => caught)
    expect(error).toMatchObject({ pendingCommandKey: key, outcomeUnknown: true })
    dismissPendingWrite(key, store)
    expect(new PendingWriteStore(storage).list()).toHaveLength(1)
  })

  it('refuses a retry after the server installation generation changes', async () => {
    const storage = new MemoryStorage()
    const store = new PendingWriteStore(storage)
    fetchMock.mockResolvedValueOnce(bootstrapResponse())
    fetchMock.mockRejectedValueOnce(new TypeError('connection dropped'))
    await expect(sendFinancialWrite(intent(), store)).rejects.toMatchObject({
      pendingCommandKey: key,
    })

    fetchMock.mockResolvedValueOnce(bootstrapResponse(newerGeneration))
    await expect(retryPendingWrite(key, store)).rejects.toBeInstanceOf(InstallationGenerationMismatchError)
    expect(fetchMock).toHaveBeenCalledTimes(3)
    expect(store.list()).toHaveLength(1)
  })

  it('treats server generation mismatch as a definite conflict and clears the command', async () => {
    const storage = new MemoryStorage()
    const store = new PendingWriteStore(storage)
    fetchMock.mockResolvedValueOnce(bootstrapResponse())
    fetchMock.mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          error: { code: 'installation_generation_mismatch', message: 'Changed', fields: [] },
          request_id: key,
        }),
        { status: 409, headers: { 'content-type': 'application/json' } },
      ),
    )

    const error = await sendFinancialWrite(intent(), store).catch((caught: unknown) => caught)
    expect(error).toMatchObject({ code: 'installation_generation_mismatch', outcomeUnknown: false })
    expect(store.list()).toHaveLength(0)
    const [, sentOptions] = fetchMock.mock.calls[1] as [string, RequestInit]
    expect(new Headers(sentOptions.headers).get('Installation-Generation')).toBe(generation)
  })

  it('does not let cancel or hide erase an unresolved command', async () => {
    const storage = new MemoryStorage()
    const store = new PendingWriteStore(storage)
    fetchMock.mockResolvedValueOnce(bootstrapResponse())
    fetchMock.mockRejectedValueOnce(new TypeError('connection dropped'))
    await expect(sendFinancialWrite(intent(), store)).rejects.toMatchObject({
      pendingCommandKey: key,
    })

    const dismissed = dismissPendingWrite(key, store)
    expect(dismissed?.idempotencyKey).toBe(key)
    expect(new PendingWriteStore(storage).get(key)?.body).toBe(exactBody)
  })

  it('keeps writes from separate tabs under independent storage keys', () => {
    const storage = new MemoryStorage()
    const firstTab = new PendingWriteStore(storage)
    const secondTab = new PendingWriteStore(storage)
    const secondKey = '537a2988-2a9e-4d49-830b-1244d5a1f01a'
    firstTab.saveNew({
      method: 'POST',
      route: '/api/accounts',
      body: exactBody,
      idempotencyKey: key,
      installationGeneration: generation,
      createdAt: '2026-11-05T10:00:00.000Z',
    })
    secondTab.saveNew({
      method: 'POST',
      route: '/api/transactions',
      body: '{"kind":"income"}',
      idempotencyKey: secondKey,
      installationGeneration: generation,
      createdAt: '2026-11-05T10:00:01.000Z',
    })

    expect(firstTab.list().map((command) => command.idempotencyKey)).toEqual([key, secondKey])
    firstTab.remove(key)
    expect(secondTab.list().map((command) => command.idempotencyKey)).toEqual([secondKey])
  })
})
