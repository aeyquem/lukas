import { expect, test } from "@playwright/test";

test("moves an envelope between two envelopes in another subcategory", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByRole("button", { name: /Presupuesto/ }).click();

  const source = page.locator('[data-envelope-id="env-rent"]');
  const target = page.locator('[data-envelope-id="env-vacation"]');

  await expect(source).toBeVisible();
  await expect(target).toBeVisible();

  await source.dragTo(target, {
    targetPosition: { x: 24, y: 2 },
  });

  const visibleEnvelopeIds = await page
    .locator("[data-envelope-id]")
    .evaluateAll((elements) =>
      elements.map((element) => element.getAttribute("data-envelope-id")),
    );

  expect(visibleEnvelopeIds).toEqual([
    "env-market",
    "env-emergency",
    "env-rent",
    "env-vacation",
  ]);
});
