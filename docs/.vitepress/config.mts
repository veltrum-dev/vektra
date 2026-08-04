import type { DefaultTheme } from "vitepress";
import { defineConfig } from "vitepress";
import { fileURLToPath } from "node:url";

const docsPublicDir = fileURLToPath(new URL("../public", import.meta.url));

function normalizeBase(value: string | undefined): string {
  const raw = (value ?? "/").trim();
  if (raw === "" || raw === "/") {
    return "/";
  }

  if (
    /^[a-z][a-z0-9+.-]*:/i.test(raw) ||
    raw.includes("\\") ||
    raw.includes("?") ||
    raw.includes("#") ||
    !/^[A-Za-z0-9._~/-]+$/.test(raw)
  ) {
    throw new Error(`VEKTRA_DOCS_BASE 不是安全的路径前缀：${raw}`);
  }

  const segments = raw.split("/").filter(Boolean);
  if (segments.some((segment) => segment === "." || segment === "..")) {
    throw new Error(`VEKTRA_DOCS_BASE 不能包含相对路径片段：${raw}`);
  }

  return `/${segments.join("/")}/`;
}

export default defineConfig({
  lang: "zh-CN",
  title: "Vektra",
  description: "为 GPUI 打造的无侵入式组件",
  base: normalizeBase(process.env.VEKTRA_DOCS_BASE),
  srcDir: "content",
  vite: {
    publicDir: docsPublicDir
  },
  locales: {
    root: {
      label: "简体中文",
      lang: "zh-CN",
      link: "/",
      title: "Vektra",
      description: "为 GPUI 打造的无侵入式组件",
      themeConfig: zhThemeConfig()
    },
    en: {
      label: "English",
      lang: "en-US",
      link: "/en/",
      title: "Vektra",
      description: "Non-invasive components for GPUI",
      themeConfig: enThemeConfig()
    }
  }
});

function zhThemeConfig(): DefaultTheme.Config {
  return {
    nav: [
      { text: "首页", link: "/" },
      { text: "快速开始", link: "/guide/getting-started" },
      { text: "组件", link: "/components/button" },
      { text: "API 参考", link: "/api/" },
      { text: "GitHub", link: "https://github.com/veltrum-dev/vektra" }
    ],
    sidebar: [
      {
        text: "指南",
        items: [
          { text: "快速开始", link: "/guide/getting-started" },
          { text: "资源与图标", link: "/guide/assets-and-icons" }
        ]
      },
      {
        text: "组件",
        items: [
          { text: "Button", link: "/components/button" },
          { text: "Checkbox", link: "/components/checkbox" },
          { text: "IconButton", link: "/components/icon-button" },
          { text: "Tooltip", link: "/components/tooltip" }
        ]
      },
      {
        text: "API 参考",
        items: [
          { text: "总览", link: "/api/" },
          { text: "Clickable", link: "/api/clickable" },
          { text: "Focusable", link: "/api/focusable" },
          { text: "Disableable", link: "/api/disableable" },
          { text: "Sizable", link: "/api/sizable" },
          { text: "回调模型", link: "/api/callbacks" },
          { text: "Vektra 公共类型", link: "/api/public-types" },
          { text: "GPUI 依赖类型", link: "/api/gpui-types" }
        ]
      }
    ],
    outline: {
      label: "页面导航",
      level: [2, 3]
    },
    docFooter: {
      prev: "上一页",
      next: "下一页"
    },
    darkModeSwitchLabel: "外观",
    lightModeSwitchTitle: "切换到浅色主题",
    darkModeSwitchTitle: "切换到深色主题",
    sidebarMenuLabel: "菜单",
    returnToTopLabel: "回到顶部"
  };
}

function enThemeConfig(): DefaultTheme.Config {
  return {
    nav: [
      { text: "Home", link: "/en/" },
      { text: "Quick Start", link: "/en/guide/getting-started" },
      { text: "Components", link: "/en/components/button" },
      { text: "API Reference", link: "/en/api/" },
      { text: "GitHub", link: "https://github.com/veltrum-dev/vektra" }
    ],
    sidebar: [
      {
        text: "Guide",
        items: [
          { text: "Quick Start", link: "/en/guide/getting-started" },
          { text: "Assets and Icons", link: "/en/guide/assets-and-icons" }
        ]
      },
      {
        text: "Components",
        items: [
          { text: "Button", link: "/en/components/button" },
          { text: "Checkbox", link: "/en/components/checkbox" },
          { text: "IconButton", link: "/en/components/icon-button" },
          { text: "Tooltip", link: "/en/components/tooltip" }
        ]
      },
      {
        text: "API Reference",
        items: [
          { text: "Overview", link: "/en/api/" },
          { text: "Clickable", link: "/en/api/clickable" },
          { text: "Focusable", link: "/en/api/focusable" },
          { text: "Disableable", link: "/en/api/disableable" },
          { text: "Sizable", link: "/en/api/sizable" },
          { text: "Callback Model", link: "/en/api/callbacks" },
          { text: "Vektra Public Types", link: "/en/api/public-types" },
          { text: "GPUI Dependency Types", link: "/en/api/gpui-types" }
        ]
      }
    ],
    outline: {
      label: "On This Page",
      level: [2, 3]
    },
    docFooter: {
      prev: "Previous page",
      next: "Next page"
    },
    darkModeSwitchLabel: "Appearance",
    lightModeSwitchTitle: "Switch to light theme",
    darkModeSwitchTitle: "Switch to dark theme",
    sidebarMenuLabel: "Menu",
    returnToTopLabel: "Return to top"
  };
}
