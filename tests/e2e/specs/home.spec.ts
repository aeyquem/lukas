import { expect, test } from "@playwright/test";

test("shows the app shell", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByText("Lukas", { exact: true })).toBeVisible();
  await expect(
    page.getByRole("heading", { level: 1, name: "Cuentas" }),
  ).toBeVisible();
});
