import { beforeEach, expect, it, vi } from 'vitest'
import { createApp, effectScope, nextTick, ref } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { PiniaColada } from '@pinia/colada'
import { useAccountListQuery, useAccountDetailQuery, useAccountActivityQuery } from '../src/features/accounts/api/queries'
import { useAccountWriteMutation, useAccountRetryMutation } from '../src/features/accounts/api/mutations'
import { apiRequest } from '../src/shared/api/client'
import { sendFinancialWrite, retryPendingWrite } from '../src/shared/api/pending-commands'
vi.mock('../src/shared/api/client', () => ({ apiRequest: vi.fn() }))
vi.mock('../src/shared/api/pending-commands', () => ({ sendFinancialWrite: vi.fn(), retryPendingWrite: vi.fn() }))
const a = '11111111-1111-4111-8111-111111111111'
const b = '22222222-2222-4222-8222-222222222222'
let app: ReturnType<typeof createApp>
beforeEach(() => {
  vi.resetAllMocks()
  const pinia = createPinia(); setActivePinia(pinia)
  app = createApp({}); app.use(pinia); app.use(PiniaColada, { pinia })
})
function setup<T>(fn: () => T) { return app.runWithContext(() => effectScope().run(fn)!) }
it('shares list requests and explicit refetch ignores freshness', async () => {
  vi.mocked(apiRequest).mockResolvedValue([])
  const [first, second] = setup(() => [useAccountListQuery(), useAccountListQuery()])
  await Promise.all([first!.refresh(true), second!.refresh(true)])
  expect(apiRequest).toHaveBeenCalledTimes(1)
  await first!.refetch(true)
  expect(apiRequest).toHaveBeenCalledTimes(2)
})
it('isolates late account detail responses and activity on account switches', async () => {
  let finish!: (v: unknown) => void
  vi.mocked(apiRequest).mockImplementation((route) => route === `accounts/${a}` ? new Promise(resolve => { finish = resolve }) : Promise.resolve({ id: b, items: [], next: null }))
  const id = ref(a)
  const [detail, activity] = setup(() => [useAccountDetailQuery(id), useAccountActivityQuery(id)])
  const old = detail!.refetch()
  id.value = b; await nextTick()
  await detail!.refetch(); await activity!.refetch()
  finish({ id: a }); await old
  expect(detail!.data.value).toMatchObject({ id: b })
  expect(apiRequest).toHaveBeenCalledWith(expect.stringContaining(`account_id=${b}`))
})
it('does not send disabled invalid detail and activity IDs', async () => {
  const [detail, activity] = setup(() => [useAccountDetailQuery('bad'), useAccountActivityQuery('')])
  await nextTick()
  expect(detail!.data.value).toBeUndefined(); expect(activity!.data.value).toBeUndefined()
  expect(apiRequest).not.toHaveBeenCalled()
})
it('retains confirmed mutation receipt when authoritative read fails', async () => {
  vi.mocked(sendFinancialWrite).mockResolvedValue({ resource_id: a, revision: 1 })
  vi.mocked(apiRequest).mockRejectedValue(new Error('read unavailable'))
  const mutation = setup(useAccountWriteMutation)
  const result = await mutation.mutateAsync({ method: 'POST', route: '/api/accounts', payload: { name: 'A' } })
  expect(result.receipt.revision).toBe(1); expect(result.readError).toBeInstanceOf(Error)
  expect(sendFinancialWrite).toHaveBeenCalledTimes(1)
})
it('explicit retry uses saved command once and separates read failure', async () => {
  vi.mocked(retryPendingWrite).mockResolvedValue({ resource_id: a, revision: 2 })
  vi.mocked(apiRequest).mockRejectedValue(new Error('read unavailable'))
  const mutation = setup(useAccountRetryMutation)
  const result = await mutation.mutateAsync({ idempotencyKey: 'saved', route: `/api/accounts/${a}` } as never)
  expect(result.receipt.revision).toBe(2); expect(result.readError).toBeInstanceOf(Error)
  expect(retryPendingWrite).toHaveBeenCalledExactlyOnceWith('saved')
})
it('does not automatically retry a rejected write', async () => {
  vi.mocked(sendFinancialWrite).mockRejectedValue(new Error('unknown outcome'))
  const mutation = setup(useAccountWriteMutation)
  await expect(mutation.mutateAsync({ method: 'POST', route: '/api/accounts', payload: {} })).rejects.toThrow('unknown outcome')
  expect(sendFinancialWrite).toHaveBeenCalledTimes(1)
  expect(apiRequest).not.toHaveBeenCalled()
})
