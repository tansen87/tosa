<script setup lang="ts">
import IconBtn from '../components/IconBtn.vue'
import { configuration as conf } from '../Configuration.ts'
import { invoke, writeClipboardText } from '../Background'
import {TranslatorStore as store} from './Store'
</script>

<template>
	<div class="flex justify-between items-center mx-4 my-1.5" data-tauri-drag-region>
		<icon-btn icon="pinup" tip="钉住窗口，不会因为失去焦点而隐藏窗口"
			:active="conf.pinup" @click="() => conf.pinup = !conf.pinup"/>
		<div class="flex items-center">
			<icon-btn icon="duplicate" :size="14" tip="复制" class="rotate-90"
						  @click="writeClipboardText(store.text.value)" />
				<icon-btn icon="clear" :size="14" tip="清空" @click="store.clear()" />
			<!-- <icon-btn icon="clipboard" tip="自动复制图片识别结果到剪切板"
				:active="conf.auto_copy"
				@click="conf.auto_copy = !conf.auto_copy"/>
			<icon-btn icon="dict" tip="仅使用词典翻译服务"
				:active="conf.only_dict"
				@click="conf.only_dict = !conf.only_dict"/> -->
			<icon-btn icon="setting" tip="设置" @click="invoke('show_setting_window')"/>
		</div>
	</div>
</template>
