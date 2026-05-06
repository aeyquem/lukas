export function formatMoney(amount: string, currency: string): string {
  const value = Number.parseFloat(amount);
  if (!Number.isFinite(value)) {
    return `${amount} ${currency}`;
  }

  return new Intl.NumberFormat("es-CL", {
    style: "currency",
    currency,
    maximumFractionDigits: currency === "CLP" ? 0 : 2,
  }).format(value);
}

export function formatDate(date: string): string {
  return new Intl.DateTimeFormat("es-CL", { dateStyle: "medium" }).format(
    new Date(`${date}T00:00:00`),
  );
}

export function currentPeriod(): string {
  return new Date().toISOString().slice(0, 7);
}
