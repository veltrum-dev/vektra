<script setup lang="ts">
import { computed } from "vue";
import { useData, withBase } from "vitepress";
import {
  Accessibility,
  AlertTriangle,
  Blocks,
  Braces,
  MonitorPlay,
  Palette,
  Zap
} from "lucide-vue-next";

const { lang } = useData();

const isEnglish = computed(() => lang.value === "en-US");

const copy = computed(() =>
  isEnglish.value
    ? {
        proofLabel: "Vektra facts",
        proof: [
          "No Root, Provider, or init",
          "Keyboard, focus, and accessible semantics built in",
          "No file I/O or JSON parsing on the render path"
        ],
        preRelease: {
          icon: AlertTriangle,
          title: "Pre-release status",
          body: "Vektra is in early development. GPUI has published pre-1.0 releases, but its API is still moving quickly and may introduce breaking changes. Vektra currently pins a specific GPUI revision and has not committed to stable compatibility with crates.io GPUI, so there is no production-ready Vektra crate release yet. Use a Git workspace or path dependency, and expect Vektra public APIs to change in breaking ways.",
          crateNote: "The vektra 0.0.1 package on crates.io only reserves the project name. It does not contain the current component library implementation and must not be used as the formal dependency."
        },
        advantagesTitle: "What Makes Vektra Different",
        advantagesLead:
          "Vektra is a pure GPUI component library for applications that already have their own lifecycle, windows, and state model.",
        primaryCards: [
          {
            icon: Blocks,
            title: "Non-invasive by default",
            body: "Components are plain GPUI elements that can be placed directly in existing Views. Your app keeps using GPUI Application, Window, View, and Context<T>.",
            points: [
              "No Vektra Root",
              "No Provider",
              "No vektra::init(cx)"
            ]
          },
          {
            icon: Accessibility,
            title: "Accessibility built into the component contract",
            body: "Button and IconButton implement concrete interaction behavior instead of leaving it as an example-page convention.",
            points: [
              "Role and accessible name",
              "Enter, Space, and mouse share the callback",
              "focus-visible and correct focus behavior",
              "disabled blocks mouse and keyboard activation"
            ]
          },
          {
            icon: Zap,
            title: "Performance-minded lightweight render path",
            body: "The default theme is parsed once and cached as Arc<ResolvedTheme>. Rendering reuses the resolved result and stays on GPUI's native SVG and asset path.",
            points: [
              "No file reads during render",
              "No JSON parsing during render",
              "Icons use GPUI SVG assets and cache"
            ]
          }
        ],
        secondaryTitle: "And still a component documentation site",
        secondaryLead:
          "The supporting pieces stay useful without competing with the core positioning.",
        secondaryCards: [
          {
            icon: Palette,
            title: "Theme tokens and modes",
            body: "Variants, sizes, Light/Dark, System mode, and interaction states come from Vektra theme tokens."
          },
          {
            icon: Braces,
            title: "Native Rust and GPUI composition",
            body: "Props, callbacks, Context<T>, Window, and the asset pipeline stay in the shape your GPUI app already uses."
          },
          {
            icon: MonitorPlay,
            title: "Docs run real Rust/GPUI WASM",
            body: "The buttons below are compiled from Rust and GPUI to WASM, not a separate HTML recreation."
          }
        ],
        labTitle: "Live Button Lab",
        labLead:
          "A compact preview that gives direct evidence for Vektra's first component surface. The ordinary variant buttons use the default MD size; the separate size row intentionally shows XS, SM, MD, and LG, so its height differences are deliberate.",
        labPreviewTitle: "Button showcase",
        codeTitle: "Same API in Rust",
        codeSnippet: `Button::new("save")
    .label("Save")
    .start_icon(IconName::Settings)
    .on_click_in(cx, |this, _, _, cx| {
        this.save_count += 1;
        cx.notify();
    });

Button::new("delete")
    .label("Delete")
    .variant(ButtonVariant::Destructive)
    .disabled(is_locked);`,
        closing: {
          title: "Start building with Vektra",
          body: "Finish the local dependency setup first, then use the Button docs to understand the component API, sizes, icon slots, and accessibility conventions.",
          navLabel: "Next steps",
          primary: "Read the quick start",
          secondary: "Browse Button",
          tertiary: "View on GitHub"
        }
      }
    : {
        proofLabel: "Vektra 事实",
        proof: [
          "无需 Root、Provider 或 init",
          "键盘、焦点与可访问语义内建",
          "render 路径无文件 I/O 或 JSON 解析"
        ],
        preRelease: {
          icon: AlertTriangle,
          title: "预发布状态",
          body: "Vektra 当前处于早期开发阶段。GPUI 虽已发布 pre-1.0 版本，但 API 仍在快速演进，可能发生破坏性变更。Vektra 当前锁定特定 GPUI revision，尚未承诺稳定兼容 crates.io GPUI，因此暂不发布可供生产使用的正式 crate。请通过 Git workspace 或 path dependency 使用，并预期 Vektra 公共 API 也可能发生破坏性变更。",
          crateNote: "crates.io 上的 vektra 0.0.1 只用于保留项目名称，不包含当前组件库实现，不能作为正式依赖。"
        },
        advantagesTitle: "Vektra 与普通组件展示站的区别",
        advantagesLead:
          "Vektra 是服务于现有 GPUI 应用的纯组件库，不接管应用生命周期、窗口模型或状态容器。",
        primaryCards: [
          {
            icon: Blocks,
            title: "无侵入式使用",
            body: "组件以普通 GPUI element 交付，可以直接作为子节点放进现有 View。宿主应用继续掌握 Application、Window、View 与 Context<T>。",
            points: [
              "不需要 Vektra Root",
              "不需要 Provider",
              "不需要 vektra::init(cx)"
            ]
          },
          {
            icon: Accessibility,
            title: "无障碍内建",
            body: "Button 与 IconButton 把交互可访问性作为组件契约实现，而不是文档里的额外约定。",
            points: [
              "Role 与可访问名称",
              "Enter、Space 和鼠标共享回调",
              "focus-visible 和正确焦点行为",
              "disabled 同时阻止鼠标与键盘激活"
            ]
          },
          {
            icon: Zap,
            title: "面向高性能的轻量渲染路径",
            body: "默认主题首次解析后缓存为 Arc<ResolvedTheme>，组件渲染阶段复用结果，并保持 GPUI 原生 SVG 与资源缓存路径。",
            points: [
              "render 阶段不读取文件",
              "render 阶段不解析 JSON",
              "Icon 使用 GPUI 原生 SVG、资源系统与缓存"
            ]
          }
        ],
        secondaryTitle: "仍然保留组件文档该有的证据",
        secondaryLead: "次级能力保持收敛，支撑核心定位，但不与三项核心价值争夺层级。",
        secondaryCards: [
          {
            icon: Palette,
            title: "主题 token 与模式",
            body: "Variant、size、Light/Dark/System 和交互状态都来自 Vektra 主题 token。"
          },
          {
            icon: Braces,
            title: "保持 Rust/GPUI 原生组合方式",
            body: "属性、回调、Context<T>、Window 与资源管线沿用现有 Rust/GPUI 形状。"
          },
          {
            icon: MonitorPlay,
            title: "文档运行真实 Rust/GPUI WASM 组件",
            body: "文档中的按钮由 Rust 和 GPUI 编译为 WASM，不是另做的 HTML 示例。"
          }
        ],
        labTitle: "实时 Button 实验台",
        labLead: "用一个紧凑预览展示 Vektra 首个组件面的直接证据。普通 variant 按钮统一使用默认 MD 尺寸；独立尺寸行刻意展示 XS、SM、MD、LG，因此高度差异是有意设计。",
        labPreviewTitle: "Button 展示",
        codeTitle: "对应的 Rust API",
        codeSnippet: `Button::new("save")
    .label("保存")
    .start_icon(IconName::Settings)
    .on_click_in(cx, |this, _, _, cx| {
        this.save_count += 1;
        cx.notify();
    });

Button::new("delete")
    .label("删除")
    .variant(ButtonVariant::Destructive)
    .disabled(is_locked);`,
        closing: {
          title: "开始使用 Vektra",
          body: "先完成本地依赖配置，再从 Button 文档了解组件 API、尺寸、图标与无障碍约定。",
          navLabel: "后续入口",
          primary: "阅读快速开始",
          secondary: "浏览 Button",
          tertiary: "在 GitHub 上查看"
        }
      }
);

const links = computed(() =>
  isEnglish.value
    ? {
        gettingStarted: withBase("/en/guide/getting-started"),
        button: withBase("/en/components/button"),
        github: "https://github.com/veltrum-dev/vektra"
      }
    : {
        gettingStarted: withBase("/guide/getting-started"),
        button: withBase("/components/button"),
        github: "https://github.com/veltrum-dev/vektra"
      }
);
</script>

<template>
  <div class="vektra-home">
    <section class="vektra-home__proof" :aria-label="copy.proofLabel">
      <span v-for="item in copy.proof" :key="item">{{ item }}</span>
    </section>

    <section class="vektra-home__pre-release" aria-labelledby="vektra-home-pre-release-title">
      <div class="vektra-home__pre-release-icon" aria-hidden="true">
        <component :is="copy.preRelease.icon" :size="22" :stroke-width="2" />
      </div>
      <div class="vektra-home__pre-release-copy">
        <h2 id="vektra-home-pre-release-title">{{ copy.preRelease.title }}</h2>
        <p>{{ copy.preRelease.body }}</p>
        <p>{{ copy.preRelease.crateNote }}</p>
      </div>
    </section>

    <section class="vektra-home__section">
      <div class="vektra-home__section-head">
        <h2>{{ copy.advantagesTitle }}</h2>
        <p>{{ copy.advantagesLead }}</p>
      </div>
      <div class="vektra-home__primary-cards">
        <article
          v-for="card in copy.primaryCards"
          :key="card.title"
          class="vektra-home__card vektra-home__card--primary"
        >
          <div class="vektra-home__icon" aria-hidden="true">
            <component :is="card.icon" :size="24" :stroke-width="1.9" />
          </div>
          <h3>{{ card.title }}</h3>
          <p>{{ card.body }}</p>
          <ul>
            <li v-for="point in card.points" :key="point">{{ point }}</li>
          </ul>
        </article>
      </div>
      <div class="vektra-home__section-head vektra-home__section-head--secondary">
        <h2>{{ copy.secondaryTitle }}</h2>
        <p>{{ copy.secondaryLead }}</p>
      </div>
      <div class="vektra-home__secondary-cards">
        <article
          v-for="card in copy.secondaryCards"
          :key="card.title"
          class="vektra-home__card vektra-home__card--secondary"
        >
          <div class="vektra-home__icon vektra-home__icon--secondary" aria-hidden="true">
            <component :is="card.icon" :size="20" :stroke-width="1.9" />
          </div>
          <h3>{{ card.title }}</h3>
          <p>{{ card.body }}</p>
        </article>
      </div>
    </section>

    <section class="vektra-home__section vektra-home__lab">
      <div class="vektra-home__section-head">
        <h2>{{ copy.labTitle }}</h2>
        <p>{{ copy.labLead }}</p>
      </div>
      <div class="vektra-home__lab-grid">
        <VektraPreview
          demo="button/showcase"
          :title="copy.labPreviewTitle"
          :height="360"
        />
        <div class="vektra-home__code">
          <h3>{{ copy.codeTitle }}</h3>
          <pre><code>{{ copy.codeSnippet }}</code></pre>
        </div>
      </div>
    </section>

    <section
      class="vektra-home__closing"
      aria-labelledby="vektra-home-closing-title"
    >
      <div class="vektra-home__closing-copy">
        <h2 id="vektra-home-closing-title">{{ copy.closing.title }}</h2>
        <p>{{ copy.closing.body }}</p>
      </div>
      <nav class="vektra-home__closing-actions" :aria-label="copy.closing.navLabel">
        <a
          class="vektra-home__closing-link vektra-home__closing-link--primary"
          :href="links.gettingStarted"
        >
          {{ copy.closing.primary }}
        </a>
        <a
          class="vektra-home__closing-link vektra-home__closing-link--secondary"
          :href="links.button"
        >
          {{ copy.closing.secondary }}
        </a>
        <a
          class="vektra-home__closing-link vektra-home__closing-link--tertiary"
          :href="links.github"
        >
          {{ copy.closing.tertiary }}
        </a>
      </nav>
    </section>
  </div>
</template>
