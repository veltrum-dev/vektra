<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useData, withBase } from "vitepress";

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

const iframeElement = ref<HTMLIFrameElement | null>(null);
const iframeSrc = ref("");
const isMounted = ref(false);
const isLoaded = ref(false);
const hasError = ref(false);

const currentTheme = computed(() => (isDark.value ? "dark" : "light"));
const currentPreviewLang = computed(() =>
  lang.value === "en-US" ? "en-US" : "zh-CN"
);
const copy = computed(() =>
  currentPreviewLang.value === "en-US"
    ? {
        openStandalone: "Open standalone",
        loading: "Loading component preview...",
        error:
          "The preview could not load. Open it standalone or check that static assets were built."
      }
    : {
        openStandalone: "在独立页面打开",
        loading: "正在加载组件预览...",
        error: "预览无法加载。请尝试在独立页面打开，或检查静态资源是否完成构建。"
      }
);

const standalonePath = computed(() =>
  previewUrl(currentTheme.value, currentPreviewLang.value)
);

const frameHeight = computed(() => {
  if (typeof props.height === "number") {
    return `${props.height}px`;
  }

  const trimmed = props.height.trim();
  return trimmed === "" ? "420px" : trimmed;
});

onMounted(() => {
  isMounted.value = true;
  iframeSrc.value = previewUrl(currentTheme.value, currentPreviewLang.value);
});

watch(currentTheme, (theme) => {
  if (!isMounted.value || iframeSrc.value === "") {
    return;
  }

  postTheme(theme);
});

function previewUrl(theme: "light" | "dark", previewLang: "zh-CN" | "en-US") {
  const encodedDemo = encodeURIComponent(props.demo);
  const encodedLang = encodeURIComponent(previewLang);
  return `${withBase("/previews/index.html")}?demo=${encodedDemo}&theme=${theme}&lang=${encodedLang}`;
}

function postTheme(theme: "light" | "dark") {
  const frame = iframeElement.value;
  const targetWindow = frame?.contentWindow;
  if (!targetWindow || iframeSrc.value === "") {
    return;
  }

  const targetOrigin = new URL(iframeSrc.value, window.location.href).origin;
  targetWindow.postMessage(
    {
      type: "vektra-preview:theme",
      value: theme
    },
    targetOrigin
  );
}

function markLoaded() {
  isLoaded.value = true;
  hasError.value = false;
  postTheme(currentTheme.value);
}

function markError() {
  isLoaded.value = true;
  hasError.value = true;
}
</script>

<template>
  <figure class="vektra-preview">
    <figcaption class="vektra-preview__header">
      <span class="vektra-preview__title">{{ title }}</span>
      <a
        class="vektra-preview__link"
        :href="standalonePath"
        target="_blank"
        rel="noreferrer"
      >
        {{ copy.openStandalone }}
      </a>
    </figcaption>
    <div class="vektra-preview__body" :style="{ height: frameHeight }">
      <div
        v-if="!isLoaded"
        class="vektra-preview__status"
        aria-live="polite"
      >
        {{ copy.loading }}
      </div>
      <div
        v-if="hasError"
        class="vektra-preview__status vektra-preview__status--error"
        role="alert"
      >
        {{ copy.error }}
      </div>
      <iframe
        v-if="iframeSrc !== ''"
        ref="iframeElement"
        class="vektra-preview__frame"
        :src="iframeSrc"
        :title="title"
        :style="{ height: frameHeight }"
        loading="lazy"
        @load="markLoaded"
        @error="markError"
      />
    </div>
  </figure>
</template>
