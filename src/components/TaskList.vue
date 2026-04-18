<script setup lang="ts">
import type { Task } from '../types/task'
import TaskItem from './TaskItem.vue'

defineProps<{ tasks: Task[] }>()
const emit = defineEmits<{ toggle: [id: string]; delete: [id: string] }>()
</script>

<template>
  <div class="task-list">
    <TransitionGroup name="list">
      <TaskItem
        v-for="task in tasks"
        :key="task.id"
        :task="task"
        @toggle="emit('toggle', task.id)"
        @delete="emit('delete', task.id)"
      />
    </TransitionGroup>
    <p v-if="tasks.length === 0" class="empty">暂无任务</p>
  </div>
</template>
