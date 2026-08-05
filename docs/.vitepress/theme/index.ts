import DefaultTheme from "vitepress/theme";
import HomePage from "./components/HomePage.vue";
import VektraExample from "./components/VektraExample.vue";
import VektraPreview from "./components/VektraPreview.vue";
import "./style.css";

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component("HomePage", HomePage);
    app.component("VektraExample", VektraExample);
    app.component("VektraPreview", VektraPreview);
  }
};
