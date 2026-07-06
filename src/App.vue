<script setup lang="ts">
import { RouterLink, RouterView } from "vue-router";
import { filesystem } from "@neutralinojs/lib";
import { onMounted } from "vue";

onMounted(() => {
  filesystem
    .readDirectory("./")
    .then((data) => {
      console.log(data);
    })
    .catch((err) => {
      console.log(err);
    });
});
</script>

<template>
  <header>
    <div class="wrapper">
      <HelloWorld msg="NeutralinoJS + Vue 3" />

      <nav>
        <RouterLink to="/">Home</RouterLink>
      </nav>
    </div>
  </header>

  <RouterView />
</template>

<style scoped></style>
