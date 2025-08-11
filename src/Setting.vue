<template>
  <v-card class="pa-6">
    <v-row align="center">
      <v-col cols="12" md="8">
        <v-text-field
          v-model="outputPath"
          label="自定义输出路径"
          placeholder="请输入输出文件夹路径"
          :rules="[rules.non_empty]"
          clearable
        />
      </v-col>
      <v-col cols="12" md="4">
        <v-btn color="primary" @click="saveOutputPath">保存路径</v-btn>
      </v-col>
    </v-row>
    <v-alert v-if="saved" type="success" class="mt-4">保存成功！</v-alert>
  </v-card>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { RULES as rules } from './common';

const outputPath = ref(localStorage.getItem('outputPath') || '');
const saved = ref(false);

function saveOutputPath() {
  if (!rules.non_empty(outputPath.value)) return;
  localStorage.setItem('outputPath', outputPath.value);
  saved.value = true;
  setTimeout(() => (saved.value = false), 1500);
}
</script>

<style scoped>

.v-card {
  max-width: none;
  margin: 20px auto;
  width: 95%;
}

</style>
