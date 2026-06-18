<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps<{
  pageNum: number
  currentPrompt: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'submit', prompt: string): void
}>()

const prompt = ref(props.currentPrompt)
const submitting = ref(false)

watch(() => props.currentPrompt, (newVal) => {
  prompt.value = newVal
})

async function handleSubmit() {
  if (!prompt.value.trim()) return
  
  submitting.value = true
  emit('submit', prompt.value)
}

function handleClose() {
  emit('close')
}
</script>

<template>
  <div class="modal-overlay" @click.self="handleClose">
    <div class="modal card">
      <div class="modal-header">
        <h3>编辑第 {{ pageNum }} 页提示词</h3>
        <button @click="handleClose" class="btn-close">×</button>
      </div>
      
      <div class="modal-body">
        <div class="form-group">
          <label>图片生成提示词</label>
          <textarea
            v-model="prompt"
            placeholder="描述你想要生成的图片内容..."
            rows="8"
          ></textarea>
        </div>
        
        <div class="tips">
          <p>💡 提示：修改提示词后会重新生成图片。</p>
        </div>
      </div>
      
      <div class="modal-actions">
        <button @click="handleClose">取消</button>
        <button 
          @click="handleSubmit" 
          :disabled="!prompt.trim() || submitting"
          class="btn-primary"
        >
          {{ submitting ? '处理中...' : '重新生成' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  width: 500px;
  max-width: 90%;
  max-height: 90vh;
  overflow-y: auto;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.modal-header h3 {
  margin: 0;
}

.btn-close {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #999;
}

.btn-close:hover {
  color: #333;
}

.form-group {
  margin-bottom: 15px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-size: 14px;
  font-weight: 500;
}

.form-group textarea {
  width: 100%;
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
  font-family: inherit;
  resize: vertical;
}

.form-group textarea:focus {
  outline: none;
  border-color: #4a9eff;
}

.tips {
  padding: 10px;
  background-color: #f0f7ff;
  border-radius: 4px;
  margin-bottom: 15px;
}

.tips p {
  margin: 0;
  font-size: 13px;
  color: #4a9eff;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.btn-primary {
  background-color: #4a9eff;
  color: #fff;
  border: none;
}

.btn-primary:hover:not(:disabled) {
  background-color: #3a8eef;
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
