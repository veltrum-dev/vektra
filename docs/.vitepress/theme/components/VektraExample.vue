<script setup lang="ts">
import { computed, onBeforeUnmount, ref, useId } from "vue";
import { useData, withBase } from "vitepress";
import { Check, CodeXml, Copy, ExternalLink } from "lucide-vue-next";
import VektraPreview from "./VektraPreview.vue";

const props = withDefaults(
  defineProps<{
    demo: string;
    title: string;
    height?: string | number;
  }>(),
  {
    height: 420
  }
);

const { isDark, lang } = useData();
const codeRegionId = `vektra-example-code-${useId().replace(/:/g, "")}`;
const codeElement = ref<HTMLElement | null>(null);
const isCodeVisible = ref(false);
const copyState = ref<"idle" | "copied" | "error">("idle");
let copyResetTimer: number | undefined;

const currentPreviewLang = computed(() =>
  lang.value === "en-US" ? "en-US" : "zh-CN"
);
const currentTheme = computed(() => (isDark.value ? "dark" : "light"));
const standalonePath = computed(() => {
  const demo = encodeURIComponent(props.demo);
  const previewLang = encodeURIComponent(currentPreviewLang.value);
  return `${withBase("/previews/index.html")}?demo=${demo}&theme=${currentTheme.value}&lang=${previewLang}`;
});
const copy = computed(() =>
  currentPreviewLang.value === "en-US"
    ? {
        showCode: "View Code",
        hideCode: "Hide Code",
        copyCode: "Copy",
        copied: "Copied",
        copyError: "Copy failed",
        openStandalone: "Open standalone"
      }
    : {
        showCode: "View Code",
        hideCode: "Hide Code",
        copyCode: "Copy",
        copied: "已复制",
        copyError: "复制失败",
        openStandalone: "在独立页面打开"
      }
);

const viewCodeLabel = computed(() =>
  isCodeVisible.value ? copy.value.hideCode : copy.value.showCode
);
const copyLabel = computed(() => {
  if (copyState.value === "copied") {
    return copy.value.copied;
  }
  if (copyState.value === "error") {
    return copy.value.copyError;
  }
  return copy.value.copyCode;
});

onBeforeUnmount(() => {
  if (copyResetTimer !== undefined) {
    window.clearTimeout(copyResetTimer);
  }
});

function toggleCode() {
  isCodeVisible.value = !isCodeVisible.value;
}

async function copyCode() {
  const source = codeElement.value
    ?.querySelector<HTMLElement>("pre code")
    ?.textContent?.replace(/\n$/, "");

  if (!source) {
    setCopyState("error");
    return;
  }

  try {
    if (navigator.clipboard) {
      await navigator.clipboard.writeText(source);
    } else if (!fallbackCopy(source)) {
      throw new Error("Clipboard API is unavailable");
    }
    setCopyState("copied");
  } catch {
    setCopyState(fallbackCopy(source) ? "copied" : "error");
  }
}

function fallbackCopy(source: string) {
  const textarea = document.createElement("textarea");
  textarea.value = source;
  textarea.setAttribute("readonly", "");
  textarea.style.position = "fixed";
  textarea.style.opacity = "0";
  document.body.appendChild(textarea);
  textarea.select();
  try {
    return document.execCommand("copy");
  } catch {
    return false;
  } finally {
    textarea.remove();
  }
}

function setCopyState(state: "copied" | "error") {
  copyState.value = state;
  if (copyResetTimer !== undefined) {
    window.clearTimeout(copyResetTimer);
  }
  copyResetTimer = window.setTimeout(() => {
    copyState.value = "idle";
    copyResetTimer = undefined;
  }, 2000);
}
</script>

<template>
  <section class="vektra-example" :aria-label="title">
    <VektraPreview
      :demo="demo"
      :title="title"
      :height="height"
      embedded
    />
    <div class="vektra-example__toolbar">
      <button
        class="vektra-example__action"
        type="button"
        :aria-controls="codeRegionId"
        :aria-expanded="isCodeVisible"
        @click="toggleCode"
      >
        <CodeXml :size="16" aria-hidden="true" />
        <span>{{ viewCodeLabel }}</span>
      </button>
      <button
        class="vektra-example__action"
        type="button"
        :aria-label="copy.copyCode"
        @click="copyCode"
      >
        <Check v-if="copyState === 'copied'" :size="16" aria-hidden="true" />
        <Copy v-else :size="16" aria-hidden="true" />
        <span>{{ copyLabel }}</span>
      </button>
      <a
        class="vektra-example__action"
        :href="standalonePath"
        target="_blank"
        rel="noopener noreferrer"
      >
        <ExternalLink :size="16" aria-hidden="true" />
        <span>{{ copy.openStandalone }}</span>
      </a>
      <span class="vektra-example__status" aria-live="polite">
        {{ copyState === "idle" ? "" : copyLabel }}
      </span>
    </div>
    <div
      v-show="isCodeVisible"
      :id="codeRegionId"
      ref="codeElement"
      class="vektra-example__code"
    >
      <slot />
    </div>
  </section>
</template>
