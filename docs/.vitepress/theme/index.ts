import DefaultTheme from "vitepress/theme";
import HomePage from "./components/HomePage.vue";
import VektraPreview from "./components/VektraPreview.vue";
import "./style.css";

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component("HomePage", HomePage);
    app.component("VektraPreview", VektraPreview);
  }
};
