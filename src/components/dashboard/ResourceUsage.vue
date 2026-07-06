<script setup lang="ts">
import { ref, onMounted } from "vue";
import Chart from "primevue/chart";

// Mock data generator for 60 minutes
const generateTimelineData = (points: number, min: number, max: number) => {
  return Array.from({ length: points }, () => Math.floor(Math.random() * (max - min + 1) + min));
};

const labels = ["-60m", "-50m", "-40m", "-30m", "-20m", "-10m", "Now"];

const cpuDataValues = generateTimelineData(7, 45, 75);
const memDataValues = generateTimelineData(7, 55, 65);

const cpuChartData = ref();
const memChartData = ref();
const chartOptions = ref();

onMounted(() => {
  // Set dark mode options based on theme
  const isDark = document.documentElement.classList.contains("my-app-dark");
  const textColor = isDark ? "#878d98" : "#7b8191";
  const borderColor = isDark ? "#2e343d" : "#d5d9e1";

  // Create canvas gradients for smoother look
  const ctxCpu = document.createElement("canvas").getContext("2d");
  const gradientCpu = ctxCpu?.createLinearGradient(0, 0, 0, 150);
  gradientCpu?.addColorStop(0, "rgba(79, 140, 255, 0.3)");
  gradientCpu?.addColorStop(1, "rgba(79, 140, 255, 0.0)");

  const ctxMem = document.createElement("canvas").getContext("2d");
  const gradientMem = ctxMem?.createLinearGradient(0, 0, 0, 150);
  gradientMem?.addColorStop(0, "rgba(142, 107, 255, 0.3)");
  gradientMem?.addColorStop(1, "rgba(142, 107, 255, 0.0)");

  cpuChartData.value = {
    labels: labels,
    datasets: [
      {
        label: "CPU Usage",
        data: cpuDataValues,
        fill: true,
        borderColor: "#4f8cff",
        backgroundColor: gradientCpu || "rgba(79, 140, 255, 0.1)",
        tension: 0.4,
        borderWidth: 2,
        pointRadius: 0,
        pointHoverRadius: 4,
      }
    ]
  };

  memChartData.value = {
    labels: labels,
    datasets: [
      {
        label: "Memory Usage",
        data: memDataValues,
        fill: true,
        borderColor: "#8e6bff",
        backgroundColor: gradientMem || "rgba(142, 107, 255, 0.1)",
        tension: 0.4,
        borderWidth: 2,
        pointRadius: 0,
        pointHoverRadius: 4,
      }
    ]
  };

  chartOptions.value = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        display: false
      },
      tooltip: {
        mode: "index",
        intersect: false,
      }
    },
    scales: {
      x: {
        grid: {
          display: false,
        },
        ticks: {
          color: textColor,
          font: {
            size: 10,
            family: "Inter"
          }
        }
      },
      y: {
        min: 0,
        max: 100,
        ticks: {
          color: textColor,
          font: {
            size: 10,
            family: "Inter"
          },
          stepSize: 50,
          callback: (value: number) => value + "%"
        },
        grid: {
          color: borderColor,
          drawTicks: false
        }
      }
    }
  };
});
</script>

<template>
  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6 shadow-sm transition-all duration-200">
    <div class="text-sm font-semibold text-[var(--text-primary)] uppercase tracking-wider mb-6">
      Resource Usage
    </div>
    
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
      <!-- CPU Column -->
      <div class="flex flex-col gap-2">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="text-xs text-[var(--text-muted)] font-medium">CPU Usage</span>
            <span class="text-2xl font-bold text-[var(--text-primary)] mt-1">68%</span>
          </div>
          <span class="text-xs text-[var(--text-muted)] font-mono">8.16 / 12 cores</span>
        </div>
        <!-- Chart Wrapper -->
        <div class="h-44 w-full mt-2">
          <Chart type="line" :data="cpuChartData" :options="chartOptions" class="h-full w-full" />
        </div>
      </div>

      <!-- Memory Column -->
      <div class="flex flex-col gap-2">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="text-xs text-[var(--text-muted)] font-medium">Memory Usage</span>
            <span class="text-2xl font-bold text-[var(--text-primary)] mt-1">61%</span>
          </div>
          <span class="text-xs text-[var(--text-muted)] font-mono">49.1 / 80 GiB</span>
        </div>
        <!-- Chart Wrapper -->
        <div class="h-44 w-full mt-2">
          <Chart type="line" :data="memChartData" :options="chartOptions" class="h-full w-full" />
        </div>
      </div>
    </div>
  </div>
</template>
