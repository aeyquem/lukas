import { expect, test } from "@playwright/test";

test("base form components stay interactive in account drawer", async ({
  page,
}) => {
  await page.goto("/");

  await page.getByLabel("Periodo").fill("2026-05");
  await expect(page.getByLabel("Periodo")).toHaveValue("2026-05");

  await page.getByRole("button", { name: "Agregar cuenta" }).click();
  await page.getByLabel("Nombre").fill("Cuenta de prueba");
  await page.getByRole("combobox", { name: "Banco" }).click();
  await page.getByRole("option", { name: "Efectivo" }).click();
  await page.getByLabel("Moneda").fill("USD");
  await page.getByLabel("Saldo inicial").fill("1500");

  await expect(page.getByLabel("Nombre")).toHaveValue("Cuenta de prueba");
  await expect(page.getByRole("combobox", { name: "Efectivo" })).toBeVisible();
  await expect(page.getByLabel("Moneda")).toHaveValue("USD");
  await expect(page.getByLabel("Saldo inicial")).toHaveValue("1500");
});

test("base workspace components stay interactive across movement and budget views", async ({
  page,
}) => {
  await page.goto("/");

  await page.getByRole("button", { name: /Movimientos/ }).click();
  await page.getByText("Moneda original").click();
  await expect(page.locator(".currency-note strong")).toHaveText(
    "moneda original",
  );

  await page.getByRole("button", { name: /Presupuesto/ }).click();
  await page.getByRole("button", { name: "Editar árbol" }).click();
  await page.getByRole("button", { name: /Renombrar Arriendo/ }).click();

  const nameInput = page.getByLabel("Nombre para Arriendo");
  await nameInput.fill("Arriendo actualizado");
  await nameInput.press("Enter");

  await expect(page.getByText("Arriendo actualizado")).toBeVisible();
});
