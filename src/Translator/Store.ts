import { ref, Ref } from 'vue'
import {
	getSize,
	hideWindow,
	invoke,
	listen,
	messageBox,
	setAlwaysOnTop, setFocus, setSize,
	writeClipboardText
} from '../Background'
import { IConfiguration, configuration } from '../Configuration.ts'
import { IDictResult } from '../types'
import TargetView from './TargetView.vue'
import { debug } from '../Logger'


class Store {
	public text: Ref<string> = ref('')

	/** 是否正在识别图片 */
	public isRecogning: Ref<boolean> = ref(false)

	public serviceEl: Map<string, InstanceType<typeof TargetView>> = new Map()

	public gorupId: string = ''

	public checkScrollHeight: Function

	async init(conf: IConfiguration) {
		const self = this

		await listen<{ key: string, value: any }>('config://updated', function(payload) {
			const { key, value } = payload
			switch (key) {
				case 'pinup':
					setAlwaysOnTop(value)
					return
			}
		})

		type IPayload = { base64: string }
		await listen<IPayload>('ocr://clip', function(payload) {
			if (self.isRecogning.value) return
			const { base64 } = payload
			self.ocrRecognize(base64, false)
		})

		window.addEventListener('blur', async () => {
			try {
				if (await invoke('active_window_is_self')) {
					return
				}
			} catch {
			}
			await setAlwaysOnTop(conf.pinup)
			if (!conf.pinup) {
				hideWindow()
			}
		})
	}

	async clear() {
		this.text.value = ''
		// this.detect_language.value = ''
		for (const [_key, target] of this.serviceEl) {
			await target.clear()
		}
	}

	public async ocrRecognize(base64: string, trans: boolean) {
		const text: string = await invoke("screenps", {
			clip: base64
		});

		this.text.value = JSON.stringify(text)
		debug('ocr: ' + JSON.stringify(text))
		
		// if (this.isRecogning.value) return
		if (!base64) {
			if (configuration.ocr_err_tip) {
				await messageBox('未找到图片数据', { title: '错误', type: 'error' })
			}
			return
		}
		// this.isRecogning.value = true
		if (!configuration.ocr_succed_show_win) {
			await invoke('show_trans_win', { focus: false })
		}
		await this.clear()
		try {
			// const text = await ocrRecognize(
			// 	ocr_services,
			// 	configuration.ocr_type,
			// 	configuration.ocr_retry_count,
			// 	base64
			// )
			
			if (!text.trim()) {
				throw new Error('未识别到文字')
			}
			if (configuration.ocr_succed_show_win) {
				await invoke('show_trans_win', { focus: false })
			}
			this.text.value = text
			// this.isRecogning.value = false
			// if (trans) {
			// 	this.translate().catch(() => {
			// 	})
			// } else {
			// 	emit('translator://focus/no-clear')
			// }
		} catch (e) {
			// this.isRecogning.value = false
			if (configuration.ocr_err_tip) {
				await messageBox(e.message, { title: '错误', type: 'error' })
			}
		}
	}

	async copyResult(res: string | IDictResult) {
		const str = (typeof res === 'string' ? res : res.text)
		if (str?.trim()) {
			await writeClipboardText(str)
			// 通知??
		}
	}

	public async resetSize() {
		const curSize = await getSize()
		let scrollHeight = document.documentElement.scrollHeight
		let height = document.documentElement.offsetHeight
		if (height > scrollHeight) {
			height = scrollHeight
		}
		// 处理超过屏幕高度的情况.....
		height += 1
		await setSize(curSize.width, Math.floor(height))
		await setFocus()
		this.checkScrollHeight && this.checkScrollHeight()
	}
}

export const TranslatorStore = new Store()