import { walletCountError, type AccountType } from './account-model'
export interface WalletDraft {
  currency: string
  opening_amount: string
  opening_date: string
  liability_clp_basis: string
}
export interface AccountDraft {
  name: string
  account_type: AccountType
  base_currency: string
  budget_eligible: boolean
  wallets: WalletDraft[]
}
export const accountTypes: {
  value: AccountType
  label: string
}[] = [
  { value: 'bank', label: 'Banco' },
  { value: 'cash', label: 'Efectivo' },
  { value: 'savings', label: 'Ahorro' },
  { value: 'card', label: 'Tarjeta' },
  { value: 'investment', label: 'Inversión' },
]
export const isoCurrency = /^[A-Z]{3}$/
const plainMoney = /^-?(0|[1-9][0-9]*)(?:\.([0-9]+))?$/
const maxMoneyMantissa = 79228162514264337593543950335n
export function isValidMoney(value: string): boolean {
  const match = plainMoney.exec(value)
  if (!match || (match[2]?.length ?? 0) > 28)
    return false
  const digits = value.replace(/^-/, '').replace('.', '')
  return BigInt(digits) <= maxMoneyMantissa
}
export function todayInSantiago(): string {
  const parts = new Intl.DateTimeFormat('en-CA', {
    timeZone: 'America/Santiago',
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).formatToParts(new Date())
  const part = (type: string) => parts.find((item) => item.type === type)?.value ?? ''
  return `${part('year')}-${part('month')}-${part('day')}`
}
export function newWallet(): WalletDraft {
  return {
    currency: 'CLP',
    opening_amount: '0',
    opening_date: todayInSantiago(),
    liability_clp_basis: '',
  }
}
export function newDraft(): AccountDraft {
  return {
    name: '',
    account_type: 'bank',
    base_currency: 'CLP',
    budget_eligible: true,
    wallets: [newWallet()],
  }
}
export function isNegativeNonZero(value: string): boolean {
  return value.startsWith('-') && !/^-(?:0+)(?:\.0+)?$/.test(value)
}
export function signOf(value: string): -1 | 0 | 1 {
  if (/^-?(?:0+)(?:\.0+)?$/.test(value))
    return 0
  return value.startsWith('-') ? -1 : 1
}

export function fieldMessage(code: string): string {
  const messages: Record<string, string> = {
    required: 'Este campo es obligatorio.',
    unsupported_value: 'Selecciona un tipo de cuenta válido.',
    must_be_iso_style_code: 'Usa un código de moneda de tres letras, por ejemplo CLP.',
    at_least_one_required: 'Añade al menos una moneda.',
    duplicate_currency: 'Esta cuenta ya tiene una billetera en esa moneda.',
    multiple_currencies_unsupported_for_account_type: 'Este tipo de cuenta admite una sola moneda. Quita las demás para guardar.',
    required_for_foreign_liability: 'Indica la base de financiamiento CLP para esta deuda.',
    must_match_liability_sign: 'La base CLP debe tener el mismo signo que la deuda.',
    only_valid_for_eligible_foreign_liability: 'Este valor solo se usa para una deuda elegible en moneda extranjera.',
    outside_decimal_envelope: 'El monto excede el rango decimal admitido.',
    stale_revision: 'La cuenta cambió. Vuelve a cargarla antes de guardar.',
  }
  return messages[code] ?? 'Revisa este valor.'
}

export function needsAccountLiabilityBasis(draft: AccountDraft, wallet: WalletDraft): boolean {
  return draft.budget_eligible && wallet.currency !== 'CLP' && isNegativeNonZero(wallet.opening_amount)
}
export function validateAccountDraft(draft: AccountDraft): Record<string, string> {
  const fieldErrors: Record<string, string> = {}
  if (!draft.name.trim())
    fieldErrors.name = 'Escribe un nombre para la cuenta.'
  if (!isoCurrency.test(draft.base_currency)) {
    fieldErrors.base_currency = 'Usa un código de moneda de tres letras, por ejemplo CLP.'
  }
  const walletError = walletCountError(draft.account_type, draft.wallets.length)
  if (walletError)
    fieldErrors.wallets = fieldMessage(walletError)
  const usedCurrencies = new Set<string>()
  draft.wallets.forEach((wallet, index) => {
    if (!isoCurrency.test(wallet.currency)) {
      fieldErrors[`wallets[${index}].currency`] = 'Usa un código de moneda de tres letras.'
    }
    else if (usedCurrencies.has(wallet.currency)) {
      fieldErrors[`wallets[${index}].currency`] = 'Esta cuenta ya tiene una billetera en esa moneda.'
    }
    usedCurrencies.add(wallet.currency)
    if (!isValidMoney(wallet.opening_amount)) {
      fieldErrors[`wallets[${index}].opening_amount`] = 'Escribe un decimal válido con hasta 28 decimales y dentro del rango permitido.'
    }
    if (!wallet.opening_date) {
      fieldErrors[`wallets[${index}].opening_date`] = 'Selecciona la fecha del saldo inicial.'
    }
    if (needsAccountLiabilityBasis(draft, wallet)) {
      if (!isValidMoney(wallet.liability_clp_basis)) {
        fieldErrors[`wallets[${index}].liability_clp_basis`] = 'Indica la base CLP firmada.'
      }
      else if (!isNegativeNonZero(wallet.liability_clp_basis)) {
        fieldErrors[`wallets[${index}].liability_clp_basis`] = 'La base CLP debe ser negativa.'
      }
    }
  })
  return fieldErrors
}
