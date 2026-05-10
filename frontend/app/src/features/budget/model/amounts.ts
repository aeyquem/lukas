import { formatMoney } from "@/common/formatters";

export const budgetCurrency = "CLP";
export const assignedAmountMaxDigits = 8;
export const assignedInputDisplaySize = 10;

export type BudgetSignedDirection = "in" | "out" | "none";

export type BudgetAmountParts = {
  number: string;
  prefix: string;
  text: string;
};

export type BudgetSignedAmountParts = BudgetAmountParts & {
  direction: BudgetSignedDirection;
};

export function amountTone(amount: string): "positive" | "negative" {
  return Number.parseFloat(amount) >= 0 ? "positive" : "negative";
}

export function assignedInputValue(amount: string): string {
  return formatAssignedInputValue(assignedInputStorageValue(amount));
}

export function assignedInputStorageValue(amount: string): string {
  return amountDigits(amount).slice(0, assignedAmountMaxDigits);
}

export function amountParts(
  amount: string,
  currency = budgetCurrency,
): BudgetAmountParts {
  const formattedAmount = formatMoney(amount, currency);
  const amountMatch = /^(\D*)(.*)$/.exec(formattedAmount);

  if (!amountMatch) {
    return {
      number: formattedAmount,
      prefix: "",
      text: formattedAmount,
    };
  }

  return {
    number: amountMatch[2],
    prefix: amountMatch[1],
    text: formattedAmount,
  };
}

export function signedSpentAmountParts(
  spentAmount: string,
): BudgetSignedAmountParts {
  const spentValue = parseAmount(spentAmount);
  const direction = spentDirection(spentValue);
  const sign = spentSign(direction);
  const parts = amountParts(formatDecimal(Math.abs(spentValue)));
  const prefix = `${sign}${parts.prefix}`;
  return {
    direction,
    number: parts.number,
    prefix,
    text: `${prefix}${parts.number}`,
  };
}

export function parseAmount(amount: string): number {
  const value = Number.parseFloat(amount);
  return Number.isFinite(value) ? value : 0;
}

export function normalizedAmount(amount: string): string {
  return formatDecimal(Math.max(0, parseAmount(amount)));
}

export function remainingAmount(
  assignedAmount: string,
  spentAmount: string,
): string {
  return formatDecimal(parseAmount(assignedAmount) - parseAmount(spentAmount));
}

export function suggestedContribution(
  targetAmount: string,
  assignedAmount: string,
  deadlineDate: string,
): number {
  const remaining = Math.max(
    0,
    parseAmount(targetAmount) - parseAmount(assignedAmount),
  );
  const months = monthsUntilDeadline(deadlineDate);
  return months <= 1 ? remaining : Math.ceil(remaining / months);
}

export function monthsUntilDeadline(deadlineDate: string): number {
  if (deadlineDate === "") {
    return 1;
  }

  const deadline = new Date(`${deadlineDate}T00:00:00`);
  if (Number.isNaN(deadline.getTime())) {
    return 1;
  }

  const today = new Date();
  const monthDelta =
    (deadline.getFullYear() - today.getFullYear()) * 12 +
    (deadline.getMonth() - today.getMonth()) +
    1;
  return Math.max(1, monthDelta);
}

export function formatDecimal(amount: number): string {
  return amount.toFixed(0);
}

function amountDigits(amount: string): string {
  return amount.replace(/\D/g, "");
}

function formatAssignedInputValue(amount: string): string {
  return amount.replace(/\B(?=(\d{3})+(?!\d))/g, ".");
}

function spentDirection(spentValue: number): BudgetSignedDirection {
  if (spentValue > 0) {
    return "out";
  }

  if (spentValue < 0) {
    return "in";
  }

  return "none";
}

function spentSign(direction: BudgetSignedDirection): string {
  if (direction === "out") {
    return "-";
  }

  if (direction === "in") {
    return "+";
  }

  return "";
}
