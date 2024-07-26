<script setup lang="ts">
import {listen, writeClipboardText } from '../Background'
import { ref } from 'vue'
import InputArea from './InputArea.vue'
import IconBtn from '../components/IconBtn.vue'
import { configuration } from '../Configuration.ts'
import {TranslatorStore as store} from './Store'

const src = ref<InstanceType<typeof InputArea>>(null)

listen('translator://focus', async function() {
	if (configuration.auto_clear) {
		await store.clear()
	}
	src.value?.focus()
})

listen('translator://focus/no-clear', () => src.value?.focus())
</script>

<template>
	<div class="flex flex-col mx-2.5 pt-1 rounded-lg bg-[--bg-box]">
		<input-area ref="src" v-model="store.text.value"/>
		<!-- <div class="flex justify-between items-center mx-2.5 my-1.5">
			<div class="flex items-center">
				<icon-btn icon="duplicate" :size="14" tip="复制" class="rotate-90"
						  @click="writeClipboardText(store.text.value)" />
				<icon-btn icon="clear" :size="14" tip="清空" @click="store.clear()" />
			</div>
		</div> -->
	</div>
</template>