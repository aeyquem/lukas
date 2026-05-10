import { describe, expect, it, vi } from "vitest";

import {
  assignedInputStorageValue,
  assignedInputValue,
  normalizedAmount,
  parseAmount,
  remainingAmount,
  signedSpentAmountParts,
  suggestedContribution,
} from "@/features/budget/model/amounts";

describe("budget amounts", () => {
  it("normalizes assigned input for storage and display", () => {
    expect(assignedInputStorageValue("$12.345.678.900")).toBe("12345678");
    expect(assignedInputValue("12345678")).toBe("12.345.678");
  });

  it("parses invalid amounts as zero and never normalizes below zero", () => {
    expect(parseAmount("not money")).toBe(0);
    expect(normalizedAmount("-100")).toBe("0");
    expect(normalizedAmount("1500.4")).toBe("1500");
  });

  it("calculates remaining and signed spent display metadata", () => {
    expect(remainingAmount("9000", "1250")).toBe("7750");
    expect(signedSpentAmountParts("1200")).toMatchObject({
      direction: "out",
      prefix: "-$",
    });
    expect(signedSpentAmountParts("-1200")).toMatchObject({
      direction: "in",
      prefix: "+$",
    });
  });

  it("suggests the remaining amount over the months until the deadline", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-05-10T12:00:00"));

    expect(suggestedContribution("12000", "3000", "2026-07-01")).toBe(3000);
    expect(suggestedContribution("12000", "3000", "")).toBe(9000);
    expect(suggestedContribution("12000", "13000", "2026-07-01")).toBe(0);

    vi.useRealTimers();
  });
});
