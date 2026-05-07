export type MovementDirection = "in" | "out";

export type LedgerMovement = {
  id: string;
  date: string;
  accountName: string;
  description: string;
  direction: MovementDirection;
  originalAmount: string;
  originalCurrency: string;
  baseAmount: string;
  baseCurrency: string;
};

export const sampleLedgerMovements: LedgerMovement[] = [
  {
    id: "movement-001",
    date: "2026-05-02",
    accountName: "Cuenta Corriente",
    description: "Sueldo mensual",
    direction: "in",
    originalAmount: "1850000",
    originalCurrency: "CLP",
    baseAmount: "1850000",
    baseCurrency: "CLP",
  },
  {
    id: "movement-002",
    date: "2026-05-03",
    accountName: "Tarjeta Visa",
    description: "Suscripcion software",
    direction: "out",
    originalAmount: "24.99",
    originalCurrency: "USD",
    baseAmount: "23740",
    baseCurrency: "CLP",
  },
  {
    id: "movement-003",
    date: "2026-05-04",
    accountName: "Efectivo",
    description: "Almuerzo",
    direction: "out",
    originalAmount: "8500",
    originalCurrency: "CLP",
    baseAmount: "8500",
    baseCurrency: "CLP",
  },
  {
    id: "movement-004",
    date: "2026-05-05",
    accountName: "Cuenta USD",
    description: "Reembolso proveedor",
    direction: "in",
    originalAmount: "110.5",
    originalCurrency: "USD",
    baseAmount: "104980",
    baseCurrency: "CLP",
  },
  {
    id: "movement-005",
    date: "2026-05-06",
    accountName: "Tarjeta Visa",
    description: "Supermercado",
    direction: "out",
    originalAmount: "68240",
    originalCurrency: "CLP",
    baseAmount: "68240",
    baseCurrency: "CLP",
  },
  {
    id: "movement-006",
    date: "2026-05-06",
    accountName: "Cuenta Corriente",
    description: "Pago arriendo",
    direction: "out",
    originalAmount: "520000",
    originalCurrency: "CLP",
    baseAmount: "520000",
    baseCurrency: "CLP",
  },
];

export function directionLabel(direction: MovementDirection): string {
  return direction === "in" ? "Entrada" : "Salida";
}
