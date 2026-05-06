import { PiniaColada } from "@pinia/colada";
import { createPinia } from "pinia";
import { createApp } from "vue";

import App from "./App.vue";
import { router } from "./router";
import "./styles/main.css";

createApp(App).use(createPinia()).use(PiniaColada).use(router).mount("#app");
