import { afterEach, describe, expect, it, vi } from "vitest";

import {
  createAccount,
  createManualTransaction,
  createTag,
  deleteTag,
  getBudgetTree,
  getMonthlyCashflow,
  getMonthlyTagReport,
  getMovements,
  getTags,
  saveBudgetTree,
  updateTag,
  updateManualTransaction,
} from "@/common/api/ledger";

describe("ledger api", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("creates accounts with the expected endpoint and body", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          id: "account-id",
          displayName: "Banco",
          type: "bank",
          currency: "CLP",
          currentBalance: "1000",
        }),
        { status: 201, headers: { "Content-Type": "application/json" } },
      ),
    );

    const account = await createAccount({
      displayName: "Banco",
      type: "bank",
      currency: "CLP",
      openingBalance: "1000",
    });

    expect(account.displayName).toBe("Banco");
    expect(fetchMock).toHaveBeenCalledWith(
      "http://127.0.0.1:5000/api/accounts",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({
          displayName: "Banco",
          type: "bank",
          currency: "CLP",
          openingBalance: "1000",
        }),
      }),
    );
  });

  it("creates manual transactions with the expected endpoint and body", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          id: "transaction-id",
          kind: "expense",
          accountId: "account-id",
          transactionDate: "2026-05-02",
          amount: "125000",
          currency: "CLP",
          accountEffect: "decrease",
          description: "Arriendo",
          categoryName: "Casa",
          tags: [
            {
              id: "tag-id",
              name: "Casa",
              normalizedName: "casa",
              color: "#64748B",
            },
          ],
        }),
        { status: 201, headers: { "Content-Type": "application/json" } },
      ),
    );

    const transaction = await createManualTransaction({
      kind: "expense",
      accountId: "account-id",
      transactionDate: "2026-05-02",
      amount: "125000",
      currency: "CLP",
      accountEffect: "decrease",
      description: "Arriendo",
      categoryName: "Casa",
      tags: ["Casa"],
    });

    expect(transaction.description).toBe("Arriendo");
    expect(fetchMock).toHaveBeenCalledWith(
      "http://127.0.0.1:5000/api/transactions",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({
          kind: "expense",
          accountId: "account-id",
          transactionDate: "2026-05-02",
          amount: "125000",
          currency: "CLP",
          accountEffect: "decrease",
          description: "Arriendo",
          categoryName: "Casa",
          tags: ["Casa"],
        }),
      }),
    );
  });

  it("updates manual transactions with the expected endpoint and body", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          id: "transaction-id",
          kind: "expense",
          accountId: "account-id",
          transactionDate: "2026-05-03",
          amount: "130000",
          currency: "CLP",
          accountEffect: "decrease",
          description: "Arriendo actualizado",
          categoryName: "Casa",
          tags: [
            {
              id: "tag-id",
              name: "Casa",
              normalizedName: "casa",
              color: "#64748B",
            },
          ],
        }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
    );

    const transaction = await updateManualTransaction({
      id: "transaction-id",
      kind: "expense",
      accountId: "account-id",
      transactionDate: "2026-05-03",
      amount: "130000",
      currency: "CLP",
      accountEffect: "decrease",
      description: "Arriendo actualizado",
      categoryName: "Casa",
      tags: ["Casa"],
    });

    expect(transaction.amount).toBe("130000");
    expect(fetchMock).toHaveBeenCalledWith(
      "http://127.0.0.1:5000/api/transactions/transaction-id",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({
          kind: "expense",
          accountId: "account-id",
          transactionDate: "2026-05-03",
          amount: "130000",
          currency: "CLP",
          accountEffect: "decrease",
          description: "Arriendo actualizado",
          categoryName: "Casa",
          tags: ["Casa"],
        }),
      }),
    );
  });

  it("requests monthly cashflow for the selected period", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          period: "2026-05",
          currency: "CLP",
          incomeTotal: "500000",
          expenseTotal: "100000",
          netCashflow: "400000",
          accounts: [],
        }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
    );

    const cashflow = await getMonthlyCashflow("2026-05");

    expect(cashflow.netCashflow).toBe("400000");
    expect(fetchMock).toHaveBeenCalledWith(
      "http://127.0.0.1:5000/api/cashflow/monthly?period=2026-05",
    );
  });

  it("requests tags", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify([
          {
            id: "tag-id",
            name: "Transport",
            normalizedName: "transport",
            color: "#64748B",
          },
        ]),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
    );

    const tags = await getTags();

    expect(tags[0]?.normalizedName).toBe("transport");
    expect(fetchMock).toHaveBeenCalledWith("http://127.0.0.1:5000/api/tags");
  });

  it("creates tags with the expected endpoint and body", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          id: "tag-id",
          name: "Transport",
          normalizedName: "transport",
          color: "#2563EB",
        }),
        { status: 201, headers: { "Content-Type": "application/json" } },
      ),
    );

    const tag = await createTag({ name: "Transport", color: "#2563EB" });

    expect(tag.color).toBe("#2563EB");
    expect(fetchMock).toHaveBeenCalledWith(
      "http://127.0.0.1:5000/api/tags",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ name: "Transport", color: "#2563EB" }),
      }),
    );
  });

  it("updates tags with the expected endpoint and body", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          id: "tag-id",
          name: "Metro",
          normalizedName: "metro",
          color: "#16A34A",
        }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
    );

    const tag = await updateTag({
      id: "tag-id",
      name: "Metro",
      color: "#16A34A",
    });

    expect(tag.name).toBe("Metro");
    expect(fetchMock).toHaveBeenCalledWith(
      "http://127.0.0.1:5000/api/tags/tag-id",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({ name: "Metro", color: "#16A34A" }),
      }),
    );
  });

  it("deletes tags with the expected endpoint", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(null, { status: 204 }),
    );

    await deleteTag("tag-id");

    expect(fetchMock).toHaveBeenCalledWith(
      "http://127.0.0.1:5000/api/tags/tag-id",
      expect.objectContaining({ method: "DELETE" }),
    );
  });

  it("requests monthly tag reports for the selected period", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          period: "2026-05",
          currency: "CLP",
          tags: [
            {
              tag: {
                id: "tag-id",
                name: "Transport",
                normalizedName: "transport",
                color: "#64748B",
              },
              incomeTotal: "0",
              expenseTotal: "12000",
              netTotal: "-12000",
              transactionCount: 1,
            },
          ],
        }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
    );

    const report = await getMonthlyTagReport("2026-05");

    expect(report.tags[0]?.expenseTotal).toBe("12000");
    expect(fetchMock).toHaveBeenCalledWith(
      "http://127.0.0.1:5000/api/reports/tags?period=2026-05",
    );
  });

  it("requests the budget tree for the selected period", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          period: "2026-05",
          categories: [],
          subcategories: [],
          envelopes: [],
          goals: [],
        }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
    );

    const tree = await getBudgetTree("2026-05");

    expect(tree.period).toBe("2026-05");
    expect(fetchMock).toHaveBeenCalledWith(
      "http://127.0.0.1:5000/api/budget?period=2026-05",
    );
  });

  it("saves the budget tree for the selected period", async () => {
    const budgetTree = {
      period: "2026-05",
      categories: [],
      subcategories: [],
      envelopes: [],
      goals: [],
    };
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify(budgetTree), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    );

    const savedTree = await saveBudgetTree("2026-05", budgetTree);

    expect(savedTree.period).toBe("2026-05");
    expect(fetchMock).toHaveBeenCalledWith(
      "http://127.0.0.1:5000/api/budget?period=2026-05",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify(budgetTree),
      }),
    );
  });

  it("requests stored movements", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify([
          {
            id: "movement-id",
            transactionId: "transaction-id",
            transactionDate: "2026-05-02",
            accountId: "account-id",
            accountName: "Banco",
            kind: "expense",
            accountEffect: "decrease",
            direction: "out",
            originalAmount: "125000",
            originalCurrency: "CLP",
            baseAmount: "125000",
            baseCurrency: "CLP",
            description: "Arriendo",
            categoryName: "Casa",
            notes: "Contrato anual",
            tags: [
              {
                id: "tag-id",
                name: "Casa",
                normalizedName: "casa",
                color: "#64748B",
              },
            ],
          },
        ]),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
    );

    const movements = await getMovements();

    expect(movements[0]?.accountName).toBe("Banco");
    expect(movements[0]?.kind).toBe("expense");
    expect(movements[0]?.accountEffect).toBe("decrease");
    expect(movements[0]?.date).toBe("2026-05-02");
    expect(movements[0]?.categoryName).toBe("Casa");
    expect(movements[0]?.notes).toBe("Contrato anual");
    expect(movements[0]?.tags[0]?.normalizedName).toBe("casa");
    expect(fetchMock).toHaveBeenCalledWith(
      "http://127.0.0.1:5000/api/movements",
    );
  });
});
