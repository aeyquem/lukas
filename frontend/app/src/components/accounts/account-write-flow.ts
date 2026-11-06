import type { PendingFinancialWrite, PendingWriteStore } from '../../shared/api/pending-commands'

export function listPendingAccountWrites(store: Pick<PendingWriteStore, 'list'>): PendingFinancialWrite[] {
  return store.list().filter(
    (command) => command.route === '/api/accounts' || command.route.startsWith('/api/accounts/'),
  )
}

/** A confirmed receipt survives a later read failure and is never resent by this flow. */
export async function writeThenRead<TReceipt, TRead>(
  write: () => Promise<TReceipt>,
  read: (receipt: TReceipt) => Promise<TRead>,
): Promise<{ receipt: TReceipt; value?: TRead; readError?: unknown }> {
  const receipt = await write()
  try {
    return { receipt, value: await read(receipt) }
  } catch (readError) {
    return { receipt, readError }
  }
}
