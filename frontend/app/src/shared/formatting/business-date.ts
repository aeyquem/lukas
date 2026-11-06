const businessDatePattern = /^(\d{4})-(\d{2})-(\d{2})$/

function utcDate(year: number, month: number, day: number): Date {
  const date = new Date(0)
  date.setUTCHours(0, 0, 0, 0)
  date.setUTCFullYear(year, month - 1, day)
  return date
}

export function assertBusinessDate(value: string): string {
  const match = businessDatePattern.exec(value)
  if (!match) throw new TypeError('Business dates must use YYYY-MM-DD.')

  const year = Number(match[1])
  const month = Number(match[2])
  const day = Number(match[3])
  const date = utcDate(year, month, day)
  if (
    date.getUTCFullYear() !== year ||
    date.getUTCMonth() !== month - 1 ||
    date.getUTCDate() !== day
  ) {
    throw new TypeError('Business date is not a valid calendar date.')
  }
  return value
}

/** Formats a date-only value at UTC midnight so viewer timezone cannot shift its day. */
export function formatBusinessDate(value: string, locale = 'es-CL'): string {
  const [year, month, day] = assertBusinessDate(value).split('-').map(Number)
  const date = utcDate(year!, month!, day!)
  return new Intl.DateTimeFormat(locale, {
    dateStyle: 'medium',
    timeZone: 'UTC',
  }).format(date)
}

/** Keeps a validated date-only value unchanged for native date inputs and API payloads. */
export function toDateInputValue(value: string): string {
  return assertBusinessDate(value)
}
