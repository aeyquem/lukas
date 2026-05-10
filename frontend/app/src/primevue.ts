import Aura from "@primeuix/themes/aura";
import type { App } from "vue";
import PrimeVue from "primevue/config";

export const primeVueOptions = {
  theme: {
    preset: Aura,
    options: {
      darkModeSelector: false,
    },
  },
  locale: {
    accept: "Si",
    apply: "Aplicar",
    clear: "Limpiar",
    contains: "Contiene",
    emptyFilterMessage: "Sin resultados",
    emptyMessage: "Sin opciones disponibles",
    equals: "Igual a",
    noFilter: "Sin filtro",
    notContains: "No contiene",
    reject: "No",
    startsWith: "Empieza con",
  },
};

export function installPrimeVue(app: App<Element>): App<Element> {
  return app.use(PrimeVue, primeVueOptions);
}
