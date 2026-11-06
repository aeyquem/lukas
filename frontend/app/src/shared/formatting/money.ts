const decimalPattern = /^(-?)(0|[1-9][0-9]*)(?:\.([0-9]+))?$/

/** Formats a wire decimal string without converting its value to a JS number. */
export function formatMoney(
  amount: string,
  currency: string,
  locale = 'es-CL',
): string {
  const match = decimalPattern.exec(amount)
  if (!match) throw new TypeError('Money must be a plain decimal string.')
  if (!/^[A-Z]{3}$/.test(currency)) throw new TypeError('Currency must be a three-letter code.')

  const [, sign, wholeDigits, fraction = ''] = match
  const magnitude = BigInt(wholeDigits)
  const signedWhole = sign === '-' ? -magnitude : magnitude
  const formatter = new Intl.NumberFormat(locale, {
    style: 'currency',
    currency,
    currencyDisplay: 'code',
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  })
  // Use a nonzero sample only to retain the negative sign for values like -0.25.
  const parts = formatter.formatToParts(
    signedWhole === 0n && sign === '-' ? -1n : signedWhole,
  )
  const lastIntegerIndex = parts.reduce(
    (lastIndex, part, index) => (part.type === 'integer' ? index : lastIndex),
    -1,
  )
  const decimalSeparator = new Intl.NumberFormat(locale)
    .formatToParts(1.1)
    .find((part) => part.type === 'decimal')?.value ?? ','

  return parts.map((part, index) => {
    if (part.type !== 'integer') return part.value

    // Intl may split a grouped value into several integer parts. Keep every
    // part and its group separators; only append the exact fraction at the end.
    const integer = signedWhole === 0n ? '0' : part.value
    return index === lastIntegerIndex && fraction.length > 0
      ? `${integer}${decimalSeparator}${fraction}`
      : integer
  }).join('')
}
