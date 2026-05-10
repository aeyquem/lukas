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
    dayNames: [
      "domingo",
      "lunes",
      "martes",
      "miércoles",
      "jueves",
      "viernes",
      "sábado",
    ],
    dayNamesMin: ["D", "L", "M", "M", "J", "V", "S"],
    dayNamesShort: ["dom", "lun", "mar", "mié", "jue", "vie", "sáb"],
    emptyFilterMessage: "Sin resultados",
    emptyMessage: "Sin opciones disponibles",
    equals: "Igual a",
    firstDayOfWeek: 1,
    monthNames: [
      "enero",
      "febrero",
      "marzo",
      "abril",
      "mayo",
      "junio",
      "julio",
      "agosto",
      "septiembre",
      "octubre",
      "noviembre",
      "diciembre",
    ],
    monthNamesShort: [
      "ene",
      "feb",
      "mar",
      "abr",
      "may",
      "jun",
      "jul",
      "ago",
      "sep",
      "oct",
      "nov",
      "dic",
    ],
    noFilter: "Sin filtro",
    notContains: "No contiene",
    reject: "No",
    startsWith: "Empieza con",
    today: "Hoy",
  },
};

export function installPrimeVue(app: App<Element>): App<Element> {
  return app.use(PrimeVue, primeVueOptions);
}
