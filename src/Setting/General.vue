<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElForm, ElFormItem, ElCheckbox, ElSelect, ElOption } from 'element-plus'
import { configuration as conf } from '../Configuration'
import HotkeyInput from './HotkeyInput.vue'
import 'element-plus/es/components/form/style/css'
import 'element-plus/es/components/form-item/style/css'
import 'element-plus/es/components/input/style/css'
import 'element-plus/es/components/input-number/style/css'
import 'element-plus/es/components/checkbox/style/css'
import 'element-plus/es/components/select/style/css'
import 'element-plus/es/components/option/style/css'
import 'element-plus/es/components/option-group/style/css'
import 'element-plus/es/components/divider/style/css'
import { isAutostart, setAutostart } from '../Background'

const isAutoStart = ref(false)


onMounted(() => {
	isAutostart().then(res => {
		isAutoStart.value = res
	})
})

async function onToggleAutoStart(v) {
	if (v) {
		try {
			await setAutostart(true)
		} catch {
			isAutoStart.value = false
		}
	} else {
		try {
			await setAutostart(false)
		} catch {
			isAutoStart.value = true
		}
	}
}
</script>

<template>
	<div class="w-full h-full p-5 overflow-y-auto">
		<el-form label-width="250px" class="m-5">
			<ElFormItem>
				<ElCheckbox v-model="isAutoStart" @change="onToggleAutoStart">开机自启动</ElCheckbox>
			</ElFormItem>
			<div class="text-[#a1a1a1] text-wrap mt-1 ml-[250px] mb-4 leading-4">
				快捷键必须包含Ctrl、Alt、Shift中的至少一个，且不能与其他软件冲突。
			</div>
			<el-form-item label="截图识别:">
				<HotkeyInput v-model="conf.screenshot_recognizer" />
				<div class="item-tip">仅截图后识别文本内容，不进行翻译。</div>
			</el-form-item>
			<el-form-item label="识别结果窗口位置">
				<ElSelect v-model="conf.win_position" placeholder="请选择翻译窗口的位置" style="width: 250px">
					<ElOption label="屏幕右上角" value="right-top" />
					<ElOption label="屏幕中间" value="center" />
					<ElOption label="鼠标位置" value="mouse" />
					<ElOption label="上次的位置" value="last" />
				</ElSelect>
			</el-form-item>

			<!-- <el-form-item label="词典模式">
				<ElCheckbox v-model="conf.only_dict">仅使用词典翻译服务</ElCheckbox>
				<div class="item-tip">
					开启后，仅显示带词典翻译的服务，若翻译失败将回退至文本翻译。
				</div>
			</el-form-item> -->

			<el-form-item label="图片识别方式">
				<ElCheckbox v-model="conf.ocr_succed_show_win">图片识别成功才显示窗口</ElCheckbox>
				<div class="item-tip">
					<p>开启后，只有当图片识别成功时，才会显示窗口。</p>
					<p>若不开启，将在截图完成后立即显示窗口。</p>
				</div>
				<ElCheckbox v-model="conf.ocr_err_tip">图片识别错误提示</ElCheckbox>
				<div class="item-tip">开启后，图片识别失败时会弹框提示错误原因</div>
			</el-form-item>

			<!-- <el-form-item label="复制结果">
				<ElCheckbox v-model="conf.auto_copy">自动复制图片识别结果到剪切板</ElCheckbox>
			</el-form-item>
			<el-form-item label="复制类型">
				<ElSelect
					v-model="conf.copy_type"
					:disabled="!conf.auto_copy"
					placeholder="请选择复制类型"
					style="width: 250px"
				>
					<ElOption label="仅复制图片识别结果" value="ocr" />
				</ElSelect>
			</el-form-item> -->
		</el-form>
	</div>
</template>
