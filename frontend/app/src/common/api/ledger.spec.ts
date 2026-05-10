import { afterEach, describe, expect, it, vi } from "vitest";

import {
  createAccount,
  createManualTransaction,
  getBudgetTree,
  getMonthlyCashflow,
  getMovements,
  saveBudgetTree,
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
    expect(fetchMock).toHaveBeenCalledWith(
      "http://127.0.0.1:5000/api/movements",
    );
  });
});
