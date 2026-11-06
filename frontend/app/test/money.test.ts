import { describe, expect, it } from 'vitest'
import { formatMoney } from '../src/shared/formatting/money'

describe('formatMoney', () => {
  it('preserves all es-CL digit groups and the exact decimal fraction', () => {
    expect(formatMoney('1234567.89', 'CLP')).toBe('CLP\u00a01.234.567,89')
  })
})
