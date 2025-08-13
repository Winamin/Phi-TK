<template>
  <v-card class="pa-6 app-card">
    <!-- grid 布局：左输入、右按钮 -->
    <div class="controls-grid">
      <div class="path-area">
        <v-text-field
          v-model="outputPath"
          label="自定义输出路径"
          placeholder="点击选择文件夹或手动输入"
          :rules="[rules.non_empty]"
          clearable
          ref="pathField"
          dense
          hint="优先使用浏览器文件系统访问 API；若需绝对路径请在 Electron 环境下运行"
          persistent-hint
          class="path-field"
          append-outer-icon="mdi-folder-open"
          @click:append-outer="selectFolder"
          aria-label="自定义输出路径"
        />

        <div v-if="selectedInfo" class="mt-2 caption" role="status" aria-live="polite">
          {{ selectedInfo }}
        </div>

        <!-- 回退 input（webkitdirectory）-->
        <input
          ref="dirInput"
          type="file"
          webkitdirectory
          directory
          multiple
          style="display: none"
          @change="onDirInputChange"
          aria-hidden="true"
        />
      </div>

      <div class="button-area" role="group" aria-label="路径操作按钮">
        <div class="button-wrapper">
          <v-btn small color="secondary" class="mr-2" @click="selectFolder">
            选择文件夹
          </v-btn>

          <v-btn small color="primary" class="mr-2" @click="saveOutputPath">
            保存路径
          </v-btn>

          <v-btn small icon class="mr-2" @click="copyPath" :disabled="!outputPath" :title="'复制路径'">
            <v-icon>mdi-content-copy</v-icon>
          </v-btn>

          <v-btn small icon @click="clearPath" :disabled="!outputPath" :title="'清除'">
            <v-icon>mdi-close</v-icon>
          </v-btn>
        </div>
      </div>
    </div>

    <v-snackbar v-model="saved" :timeout="1500" color="success" top right>
      保存成功！
    </v-snackbar>

    <v-alert v-if="warning" type="warning" class="mt-4">
      {{ warning }}
    </v-alert>
  </v-card>
</template>

<script setup lang="ts">
// 显式声明组件名以满足 ESLint 的 multi-word 要求
// 需要 Vue SFC 宏支持（Vue 3.3+ / Vite 默认支持）
defineOptions({ name: 'SettingsPanel' });

import { ref } from 'vue';
import { RULES as rules } from './common';

const outputPath = ref<string>(localStorage.getItem('outputPath') || '');
const saved = ref(false);
const warning = ref('');
const selectedInfo = ref<string | null>(null);

const dirInput = ref<HTMLInputElement | null>(null);
const pathField = ref<HTMLElement | null>(null);

// 目录句柄（可选）
let dirHandle: any = null;
const pickerAvailable = 'showDirectoryPicker' in window;

/**
 * 选择文件夹（由用户点击触发）
 */
function selectFolder() {
  warning.value = '';
  selectedInfo.value = null;

  if (pickerAvailable) {
    // 使用 then/catch 保证是在用户激活下调用
    // @ts-ignore
    (window as any).showDirectoryPicker()
      .then((handle: any) => {
        if (!handle) return;
        dirHandle = handle;
        outputPath.value = handle.name || '已选择文件夹';
        selectedInfo.value = `已选择文件夹：${handle.name}（浏览器不暴露绝对路径）`;
      })
      .catch((err: any) => {
        if (err && err.name !== 'AbortError') {
          warning.value = '选择文件夹时出错：' + (err.message || err);
        }
      });
  } else {
    // 回退：触发隐藏的 input[file webkitdirectory]
    dirInput.value?.click();
  }
}

/**
 * webkitdirectory 回退处理：从 webkitRelativePath 推断根目录名并显示文件数
 */
function onDirInputChange(e: Event) {
  const input = e.target as HTMLInputElement;
  const files = input.files;
  if (!files || files.length === 0) return;

  const first = files[0] as any;
  const rel = first?.webkitRelativePath as string | undefined;

  let root = '';
  if (rel) {
    root = rel.split('/')[0] || first.name;
  } else {
    root = files[0].name;
  }

  outputPath.value = root;
  selectedInfo.value = `回退选择：${files.length} 个文件（根：${root}）`;
  input.value = '';
}

/**
 * 保存到 localStorage（只保存可读字符串）
 */
function saveOutputPath() {
  if (!rules.non_empty(outputPath.value)) {
    warning.value = '路径不能为空';
    return;
  }
  localStorage.setItem('outputPath', outputPath.value);
  saved.value = true;
  setTimeout(() => (saved.value = false), 1500);
}

/**
 * 清除
 */
function clearPath() {
  outputPath.value = '';
  selectedInfo.value = null;
  dirHandle = null;
  localStorage.removeItem('outputPath');
}

/**
 * 复制到剪贴板
 */
async function copyPath() {
  if (!outputPath.value) return;
  try {
    await navigator.clipboard.writeText(outputPath.value);
    saved.value = true;
    setTimeout(() => (saved.value = false), 1500);
  } catch (err) {
    warning.value = '复制失败：请手动复制';
  }
}
</script>

<style scoped>
.app-card {
  max-width: 980px;
  margin: 18px auto;
  width: 95%;
  background-color: #003366;
  color: #ffffff;
  box-shadow: 0 6px 18px rgba(0,0,0,0.25);
  border-radius: 12px;
}

/* grid：左侧最小宽 260px，右侧为按钮区 */
.controls-grid {
  display: grid;
  grid-template-columns: minmax(260px, 1fr) auto;
  gap: 12px;
  align-items: center;
}

.path-area .path-field {
  width: 100%;
  box-sizing: border-box;
}

.button-area {
  display: flex;
  justify-content: flex-end;
  align-items: center;
}

.button-wrapper {
  display: flex;
  gap: 8px;
  flex-wrap: nowrap;
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  padding-bottom: 2px;
}

@media (max-width: 720px) {
  .controls-grid {
    grid-template-columns: 1fr;
  }

  .button-area {
    margin-top: 8px;
    justify-content: flex-start;
  }

  .button-wrapper {
    flex-wrap: wrap;
    overflow-x: visible;
  }
}

/* 按钮配色 */
.v-btn.primary {
  background-color: #1e88e5 !important;
}
.v-btn.secondary {
  background-color: #00509e !important;
  color: #fff !important;
}

.caption {
  color: rgba(255,255,255,0.85);
}
</style>
