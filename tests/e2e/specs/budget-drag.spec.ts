import { expect, test } from "@playwright/test";

test("moves an envelope between two envelopes in another subcategory", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByRole("button", { name: /Presupuesto/ }).click();

  const source = page
    .locator("[data-envelope-id]")
    .filter({ hasText: "Arriendo" })
    .first();
  const target = page
    .locator("[data-envelope-id]")
    .filter({ hasText: "Vacaciones" })
    .first();

  await expect(source).toBeVisible();
  await expect(target).toBeVisible();

  const sourceId = await source.getAttribute("data-envelope-id");

  await source.dragTo(target, {
    targetPosition: { x: 24, y: 2 },
  });

  await expect(
    page.locator(`[data-envelope-id="${sourceId ?? ""}"]`),
  ).toBeVisible();
});
