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
  const today = new Date();
  const year = today.getFullYear().toString();
  const month = (today.getMonth() + 1).toString().padStart(2, "0");
  return `${year}-${month}`;
}
