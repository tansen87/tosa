<script setup lang="ts">
import { onMounted, ref, computed, nextTick } from 'vue'
import IconBtn from '../components/IconBtn.vue'
import { TranslatorStore as store } from './Store'
import { IDictResult, ITransServiceConfig } from '../types'
import 'animate.css'

type ITransResult = string | IDictResult
const props = defineProps({
	config: { type: Object, required: true }
})
const expand = ref(false)
const state = ref(false)
const isLoading = ref(false)
const elapsed_time = ref(0)
const config = computed(() => props.config as ITransServiceConfig)
const trans_result = ref<ITransResult>('')

onMounted(async () => {
	await store.resetSize()
	if (store.text.value) {
		translate().catch(() => {})
	}
})

function toggleExpand() {
	expand.value = !expand.value
	store.resetSize()
}

async function onClear(){
	if (isLoading.value) return
	elapsed_time.value = 0
	state.value = false
	trans_result.value = ''
	expand.value = false
	await nextTick()
	await store.resetSize()
}

async function translate(cache = true): Promise<{ id: string, data: ITransResult }> {
	const text = store.text.value
	const result: { id: string, data: ITransResult } = { id: config.value.id || '', data: '' }
	const service = config.value.service
	if (isLoading.value) return result
	elapsed_time.value = 0
	isLoading.value = true
	state.value = false
	trans_result.value = ''
	if (!service || !service.Translate || !text.trim()) {
		expand.value = false
		isLoading.value = false
		return result
	}
	const st = Date.now()
	elapsed_time.value = Date.now() - st
	isLoading.value = false
	expand.value = true
	await nextTick()
	await store.resetSize()
	return result
}

defineExpose({
	translate,
	clear: onClear
})
</script>

<template>
	<div class="flex flex-col bg-[var(--bg-box)] rounded-lg overflow-hidden">
		<div class="flex justify-between items-center h-[35px] text-[var(--text-color)] cursor-pointer mx-3"
			:class="expand ? 'rounded-t-lg' : 'rounded-lg'" @click="toggleExpand">
			<div class="flex items-center">
				<img :src="config.service.icon" class="mr-2 w-4 h-4" />
				{{ config.label }}
				<svg v-if="isLoading" class="animate-spin ml-2 h-5 w-5 text-[var(--placeholder)]" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
				</svg>
				<div v-if="elapsed_time > 0" class="text-sm text-[var(--text-color)] ml-2">
					({{ elapsed_time }}ms)
				</div>
			</div>
			<icon-btn icon="expand" :size="12" class="mr-1.5" :class="!expand && 'rotate-180'" @click.prevent.stop="toggleExpand" />
		</div>
	</div>
</template>