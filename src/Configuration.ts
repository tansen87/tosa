import { reactive, watch, UnwrapNestedRefs } from 'vue'
import { IServiceConfig, ITransServiceConfig } from './types'
import { debug } from './Logger.ts'
import { plugins as TransPlugins } from './Plugins/Translator'
import { deepEqual } from './Utils.ts'
import { invoke, listen, getLabel } from './Background'


export type OcrType = 'round' | 'concurrent' | 'first'
export type DetectType = 'local' | 'order' | 'concurrent' | 'concurrent_most' | string
export  type WinPosition = 'right-top' | 'center' | 'last' | 'mouse'
export class Configuration {
	/** 是否钉住窗口 */
	public pinup: boolean = false
	/** 截图识别快捷键 */
	public screenshot_recognizer = ''

	/** 图片识别成功才显示窗口 */
	public ocr_succed_show_win = false
	/** 图片识别错误提示 */
	public ocr_err_tip = true

	/** 是否只使用词典服务,为false有bug */
	public only_dict = true
	/** 是否自动清空内容 */
	public auto_clear = false
	/** 自动复制内容到剪切板 */
	public auto_copy = true
	/** 首选的翻译服务 */
	public copy_type: 'first' | string = ''

	/** 窗口位置 */
	public win_position: WinPosition = 'right-top'

	public trans_services: IServiceConfig[] = []
}
export type IConfiguration = UnwrapNestedRefs<Configuration>

export const configuration: IConfiguration = reactive(new Configuration())
let initConfiguration = false

function merge(target?: Configuration, source?: Configuration) {
	if (!target || !source) return
	for (const key in target) {
		const source_value = source[key]
		const target_value = target[key]
		if (source_value === undefined || typeof source_value !== typeof target_value) {
			continue
		}
		if (Array.isArray(target_value)) {
			target[key] = source_value
		} else if (typeof target_value === 'object') {
			merge(target[key], source_value)
		} else {
			target[key] = source_value
		}
	}
}

export async function useConfig(defaultConfig?: Record<string, any>) {
	if (initConfiguration) return configuration
	//  合并默认配置
	merge(configuration, defaultConfig as any)
	try {
		// 获取后台配置
		const file_config = await invoke<Record<string, any>>('get_config')
		if (file_config) {
			for (const key in file_config) {
				const value = file_config[key]
				if (!deepEqual(configuration[key], value)) {
					configuration[key] = value
				}
			}
		}
	} catch (e){
		debug(`load app config error: {}`, e)
	}

	let last = JSON.parse(JSON.stringify(configuration)) as Record<string, any>
	let stopWatch: Function = undefined
	function startWatch() {
		stopWatch = watch(() => configuration, async (newValue) => {
			for (const key of Object.keys(last)) {
				const oldValue = last[key] as any
				const v = newValue[key] as any
				if (!deepEqual(oldValue, v)) {
					await invoke('set_config_by_key', { key, value: v })
				}
			}
			last = JSON.parse(JSON.stringify(newValue)) as Record<string, any>
		}, { deep: true })
	}
	startWatch()

	await listen<{ key: string, value: any }>('config://updated', async (payload, windowLabel) => {
		if (windowLabel === getLabel()) return
		const { key, value } = payload
		if (value === null || value === undefined) return
		stopWatch()
		configuration[key] = value
		last = JSON.parse(JSON.stringify(configuration)) as Record<string, any>
		startWatch()
	})

	initConfiguration = true
	return configuration
}

export function generateTransConfig(item: IServiceConfig): ITransServiceConfig | undefined {
	if (!item.enable) return undefined

	const plugin = TransPlugins.find(x => x.name === item.name)
	if (!plugin) return undefined
	const temp = JSON.parse(JSON.stringify(item)) as ITransServiceConfig
	if (!temp.label) {
		temp.label = plugin.label || plugin.name
	}
	if (configuration.only_dict) {
		if (!plugin.Dict){
			return undefined
		}
	} else if (!item.transVerify && !plugin.Dict) return undefined
	temp.service = plugin
	return temp
}