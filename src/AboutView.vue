<template>
  <div class="pa-8 w-100 h-100 d-flex flex-column align-center glass-background" style="max-width: 1280px; gap: 1rem">
    <h1 class="gradient-text" v-t="'app'"></h1>
    <v-scale-transition>
      <h4 class="mt-n2 text-glow">v{{ appVersion }}</h4>
    </v-scale-transition>
    <v-btn 
      prepend-icon="mdi-github" 
      color="accent"
      variant="flat"
      class="elevation-8 hover-scale"
      @click="open('https://github.com/Winamin/Phi-TK.git')"
    >GitHub</v-btn>
    <p class="text-gradient">Licensed by GPLv3</p>
  </div>
</template>

<style scoped>
.glass-background {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(12px);
  border-radius: 16px;
}

.gradient-text {
  background: linear-gradient(45deg, #2196F3, #E91E63);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.text-glow {
  text-shadow: 0 0 8px rgba(33, 150, 243, 0.5);
}

.hover-scale {
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.hover-scale:hover {
  transform: scale(1.05);
}

.text-gradient {
  background: linear-gradient(45deg, #4CAF50, #FFEB3B);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
</style>