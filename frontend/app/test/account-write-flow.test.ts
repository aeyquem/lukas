import { describe, expect, it, vi } from 'vitest'
import { listPendingAccountWrites, writeThenRead } from '../src/features/accounts/api/write-flow'
import { PendingWriteStore, type PendingFinancialWrite } from '../src/shared/api/pending-commands'

class MemoryStorage implements Storage {
  private values = new Map<string, string>()
  get length(): number { return this.values.size }
  clear(): void { this.values.clear() }
  getItem(key: string): string | null { return this.values.get(key) ?? null }
  key(index: number): string | null { return [...this.values.keys()][index] ?? null }
  removeItem(key: string): void { this.values.delete(key) }
  setItem(key: string, value: string): void { this.values.set(key, value) }
}

function command(idempotencyKey: string, route: string): PendingFinancialWrite {
  return {
    method: 'POST',
    route,
    body: '{"name":"Cuenta"}',
    idempotencyKey,
    installationGeneration: 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
    createdAt: '2026-11-05T12:00:00.000Z',
  }
}

describe('account write flow', () => {
  it('discovers persisted account commands after constructing a fresh store', () => {
    const storage = new MemoryStorage()
    const originalStore = new PendingWriteStore(storage)
    originalStore.saveNew(command('11111111-1111-4111-8111-111111111111', '/api/accounts'))
    originalStore.saveNew(command('22222222-2222-4222-8222-222222222222', '/api/accounts/account-id'))
    originalStore.saveNew(command('33333333-3333-4333-8333-333333333333', '/api/transactions'))

    const reloadedCommands = listPendingAccountWrites(new PendingWriteStore(storage))

    expect(reloadedCommands.map((pending) => pending.route)).toEqual([
      '/api/accounts',
      '/api/accounts/account-id',
    ])
  })

  it('keeps a confirmed receipt when its follow-up read fails and never repeats the write', async () => {
    const receipt = { resource_id: 'account-123', revision: 1 }
    const write = vi.fn().mockResolvedValue(receipt)
    const read = vi.fn().mockRejectedValue(new Error('read unavailable'))

    const result = await writeThenRead(write, read)

    expect(result.receipt).toBe(receipt)
    expect(result.readError).toEqual(new Error('read unavailable'))
    expect(write).toHaveBeenCalledTimes(1)
    expect(read).toHaveBeenCalledTimes(1)
  })
})
