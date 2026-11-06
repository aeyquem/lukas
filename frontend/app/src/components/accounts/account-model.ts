export type AccountType = 'bank' | 'cash' | 'savings' | 'card' | 'investment'

export function canAddCurrencyWallet(accountType: AccountType): boolean {
  return accountType === 'card' || accountType === 'cash'
}

export function setDraftAccountType<T>(
  draft: { account_type: AccountType; wallets: T[] },
  accountType: AccountType,
): void {
  draft.account_type = accountType
}

export function walletCountError(accountType: AccountType, count: number): string | undefined {
  if (count < 1) return 'at_least_one_required'
  if (accountType !== 'card' && accountType !== 'cash' && count > 1) {
    return 'multiple_currencies_unsupported_for_account_type'
  }
  return undefined
}
