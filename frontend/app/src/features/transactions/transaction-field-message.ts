const fieldMessages: Record<string, string> = {
  required_for_opening_date: 'Elige cómo tratar el movimiento respecto del saldo inicial.',
  unsupported: 'Selecciona una opción válida.',
  required: 'Completa este campo.',
  must_be_positive: 'Escribe un monto mayor que cero.',
  same_currency_amounts_must_match: 'Los montos deben coincidir para monedas iguales.',
  must_differ: 'Elige una billetera distinta.',
  required_for_cross_currency: 'Añade evidencia de conversión para esta transferencia.',
  unknown_or_archived: 'La categoría ya no está disponible.',
  must_match_source_expense: 'El reembolso debe seguir la categoría del gasto original.',
  refund_only: 'Solo un reembolso puede vincularse con un gasto.',
  cannot_link_to_self: 'El movimiento no puede vincularse consigo mismo.',
  stale_revision: 'El movimiento cambió. Recarga su versión actual antes de corregirlo.',
}

export function transactionFieldMessage(code: string): string {
  return fieldMessages[code] ?? 'Revisa este campo.'
}
