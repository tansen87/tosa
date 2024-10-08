<template>
	<div>
		<!-- 显示 -->
		<canvas id="main_photo"></canvas>
		<canvas id="clip_photo"></canvas>
		<!-- 大小栏 -->
		<div id="clip_wh">
			<div id="wh" contenteditable="true" title="大小"></div>
		</div>
		<!-- 鼠标栏 -->
		<div id="mouse_bar" style="display: none;">
			<div id="point_color"></div>
			<div id="clip_copy"></div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import {
	listen,
	emit,
	writeClipboardText,
	hideWindow,
	showWindow,
	setAlwaysOnTop,
	setFullscreen,
	setFocus
} from '../Background'
import { debug } from '../Logger'
import Color from 'color'

onMounted(() => {
	const main_canvas = document.getElementById('main_photo') as HTMLCanvasElement
	const clip_canvas = document.getElementById('clip_photo') as HTMLCanvasElement
	const mouse_bar_div = document.getElementById('mouse_bar') as HTMLDivElement
	const point_color = document.getElementById('point_color') as HTMLDivElement
	const clip_copy = document.getElementById('clip_copy') as HTMLDivElement
	const clip_wh = document.getElementById('clip_wh') as HTMLDivElement
	const clip_wh_div = document.getElementById('wh') as HTMLDivElement
	const clip_ctx = clip_canvas.getContext('2d')
	const color_size = 15
	const color_i_size = 10
	let final_rect = [0, 0, main_canvas.width, main_canvas.height]
	let ratio = window.devicePixelRatio
	let selecting = false
	let canvas_rect = []
	let in_rect = false
	let moving = false
	let oe: any = ''
	let o_final_rect: number[] | undefined = undefined
	let direction

	document.documentElement.style.setProperty('--color-size', `${color_size * color_i_size}px`)
	document.documentElement.style.setProperty('--color-i-size', `${color_i_size}px`)
	document.documentElement.style.setProperty('--color-i-i', `${color_size}`)

	main_canvas.style.width = window.screen.width + 'px'
	clip_canvas.style.width = window.screen.width + 'px'
	// 第一次截的一定是桌面,所以可提前定义
	main_canvas.width = clip_canvas.width = window.screen.width * window.devicePixelRatio
	main_canvas.height = clip_canvas.height = window.screen.height * window.devicePixelRatio
	// 生成取色器
	let inner_html = ''
	for (let i = 1; i <= color_size ** 2; i++) {
		if (i === (color_size ** 2 + 1) / 2) {
			// 光标中心点
			inner_html += `<span id="point_color_t_c"></span>`
		} else {
			inner_html += `<span id="point_color_t"></span>`
		}
	}
	point_color.innerHTML = inner_html
	inner_html = null
	const point_color_span_list = document.querySelectorAll('#point_color > span') as NodeListOf<HTMLSpanElement>
	let action_type = ''
	listen<{ name: string, base64: string }>('screenshot://capture', (payload) => {
		if (!payload) return
		const { name, base64 } = payload
		if (!name || !base64) return
		action_type = name
		const img = new Image()
		img.src = 'data:image/png;base64,' + base64
		img.onerror = function() {
			debug('invoke image load base64 error')
			winClose()
		}
		img.onload = async function() {
			main_canvas.width = clip_canvas.width = img.width
			main_canvas.height = clip_canvas.height = img.height
			main_canvas.getContext('2d').drawImage(img, 0, 0)
			final_rect = [0, 0, img.width, img.height]
			draw_clip_rect()
			// 复制大小和颜色
			point_color.style.height = ''
			clip_copy.className = 'clip_copy_h'
			mouse_bar_div.style.pointerEvents = 'none'
			await showWindow()
			await setAlwaysOnTop(true)
			await setFullscreen(true)
			await setFocus()
		}
	})

	function winClose() {
		final_rect = [0, 0, main_canvas.width, main_canvas.height]
		o_final_rect = undefined
		direction = 'none'
		selecting = in_rect = moving = false
		clip_canvas.style.cursor = 'move'
		hideWindow()
	}

	document.addEventListener('keydown', function(ev) {
		if (ev.keyCode === 27) { // esc
			winClose()
		} else if (ev.keyCode === 65) { //  a
			clearRect()
		} else if (ev.keyCode === 72 || ev.keyCode === 75) { //  h or k
			writeClipboardText(clip_copy.innerHTML)
			debug('copy color: ' + clip_copy.innerHTML)
			winClose()
		}
	})

	clip_canvas.ondblclick = async (e) => {
		is_in_clip_rect(e)
		if (!in_rect) return
		debug('invoke clip double click ok:' + action_type)
		await emit('ocr://clip', { base64: get_clip_photo(), action: action_type })
		winClose()
	}

	function clearRect() {
		final_rect = [0, 0, main_canvas.width, main_canvas.height]
		clip_canvas.style.cursor = 'move'
		direction = 'none'
		draw_clip_rect()
	}

	clip_canvas.onmousedown = (e) => {
		if (e.button !== 0) return
		e.stopPropagation()
		e.preventDefault()
		is_in_clip_rect(e)
		if (in_rect) {
			//鼠标在选框中
			is_in_clip_rect(e)
			oe = e
			o_final_rect = final_rect
			moving = true
			move_rect(o_final_rect, oe, oe)
		} else {
			//不在选框中
			selecting = true
			canvas_rect = [e.offsetX, e.offsetY] // 用于框选
			final_rect = p_xy_to_c_xy(canvas_rect[0], canvas_rect[1], e.offsetX, e.offsetY)
		}
	}

	clip_canvas.onmousemove = (e) => {
		if (e.button === 0 && selecting) {
			// 画框
			final_rect = p_xy_to_c_xy(canvas_rect[0], canvas_rect[1], e.offsetX, e.offsetY)
			draw_clip_rect()
		}
		if (!selecting && !moving) {
			is_in_clip_rect(e)
		}
		moving && move_rect(o_final_rect, oe, e)
	}
	document.addEventListener('contextmenu', (e) => {
		e.preventDefault()
	})
	clip_canvas.onmouseup = (e) => {
		e.stopPropagation()
		e.preventDefault()
		if (e.button === 0) {
			if (!in_rect) {
				clip_ctx.closePath()
				selecting = false
				final_rect = p_xy_to_c_xy(canvas_rect[0], canvas_rect[1], e.offsetX, e.offsetY)
				if (final_rect[2] > 15 && final_rect[3] > 15) {
					draw_clip_rect()
				} else {
					clearRect()
				}
			} else if (moving) {
				move_rect(o_final_rect, oe, e)
				moving = false
				o_final_rect = undefined
			}
		} else {
			const is_not_draw_clip_rect = final_rect[2] === main_canvas.width && final_rect[3] === main_canvas.height
			if (!in_rect && is_not_draw_clip_rect) {
				winClose()
			} else {
				clearRect()
			}
		}
	}

	// 鼠标栏实时跟踪
	document.onmousemove = (e: MouseEvent) => {
		if (clip_canvas.offsetWidth !== 0) {
			// 鼠标位置文字
			const c_rect = (e.target as HTMLDivElement).getBoundingClientRect()
			const c_x = e.offsetX + c_rect.left
			const c_y = e.offsetY + c_rect.top
			const pos = p_xy_to_c_xy(c_x, c_y, c_x, c_y)
			// 鼠标跟随栏
			mouse_bar(final_rect, pos[0], pos[1])
		}
		// 鼠标跟随栏
		let x = e.clientX + 16
		let y = e.clientY + 16
		const w = mouse_bar_div.offsetWidth
		const h = mouse_bar_div.offsetHeight
		if (x + w > window.screen.width) {
			x = x - w - 32
		}
		if (y + h > window.screen.height) {
			y = y - h - 32
		}
		mouse_bar_div.style.left = `${x}px`
		mouse_bar_div.style.top = `${y}px`
	}

	// 初始化鼠标栏
	document.onmouseenter = () => {
		mouse_bar_div.style.display = 'flex'
	}

	// 画框(遮罩)
	function draw_clip_rect() {
		clip_ctx.clearRect(0, 0, clip_canvas.width, clip_canvas.height)
		clip_ctx.beginPath()
		// 框选为黑色遮罩
		clip_ctx.fillStyle = '#0008'
		clip_ctx.fillRect(0, 0, clip_canvas.width, final_rect[1])
		clip_ctx.fillRect(0, final_rect[1], final_rect[0], final_rect[3])
		clip_ctx.fillRect(final_rect[0] + final_rect[2], final_rect[1], clip_canvas.width - (final_rect[0] + final_rect[2]), final_rect[3])
		clip_ctx.fillRect(0, final_rect[1] + final_rect[3], clip_canvas.width, clip_canvas.height - (final_rect[1] + final_rect[3]))
		clip_ctx.fillStyle = '#0000'
		clip_ctx.fillRect(final_rect[0], final_rect[1], final_rect[2], final_rect[3])
		// 大小栏
		wh_bar(final_rect)
	}

	// 大小栏
	function wh_bar(final_rect) {
		clip_wh.style.right = ``
		// 位置
		const dw = clip_wh.offsetWidth
		const dh = clip_wh.offsetHeight
		let x = 0
		if (dw >= final_rect[2] / ratio) {
			if (dw + final_rect[0] <= main_canvas.offsetWidth) {
				x = final_rect[0] / ratio // 对齐框的左边
				clip_wh.style.left = `${x}px`
			} else {
				clip_wh.style.right = `0px`
			}
		} else {
			x = final_rect[0] / ratio + final_rect[2] / ratio / 2 - dw / 2
			clip_wh.style.left = `${x}px`
		}
		let y
		if (final_rect[1] - (dh * ratio + 10) >= 0) {
			y = final_rect[1] - (dh * ratio + 10) // 不超出时在外
		} else {
			y = final_rect[1] + 10
		}
		clip_wh.style.top = `${y / ratio}px`
		setWhHtml()
	}

	function setWhHtml() {
		clip_wh_div.innerHTML = `${final_rect[2]} × ${final_rect[3]}`
	}

	function resizeWhBox() {
		let l = clip_wh_div.innerHTML.split(/[,×]/)
		if (l.length !== 2) {
			return setWhHtml()
		}
		const xy = []
		for (let n of l) {
			const num = Number(n)
			if (Number.isNaN(num)) {
				return setWhHtml()
			}
			xy.push(num)
		}
		final_rect[2] = xy[0]
		final_rect[3] = xy[1]
		final_rect_fix()
		draw_clip_rect()
	}

	clip_wh.onkeydown = (e) => {
		if (e.key === 'Enter') {
			e.preventDefault()
			resizeWhBox()
		}
	}
	clip_wh_div.onblur = () => resizeWhBox()

	// 鼠标跟随栏
	function mouse_bar(final_rect, x, y) {
		const x0 = final_rect[0],
			x1 = final_rect[0] + final_rect[2],
			y0 = final_rect[1],
			y1 = final_rect[1] + final_rect[3]
		const color = main_canvas.getContext('2d').getImageData(
			x - (color_size - 1) / 2,
			y - (color_size - 1) / 2,
			color_size,
			color_size
		).data  // 取色器密度
		// 分开每个像素的颜色
		for (let i = 0, len = color.length; i < len; i += 4) {
			const color_g = color.slice(i, i + 4)
			color_g[3] /= 255
			const ii = parseInt((i / 4) + '')
			const xx = (ii % color_size) + (x - (color_size - 1) / 2)
			const yy = parseInt((ii / color_size) + '') + (y - (color_size - 1) / 2)
			if (!(x0 <= xx && xx <= x1 - 1 && y0 <= yy && yy <= y1 - 1) && ii !== (color.length / 4 - 1) / 2) {
				// 框外
				point_color_span_list[ii].id = 'point_color_t_b'
				point_color_span_list[ii].style.background = `rgba(${color_g[0]}, ${color_g[1]}, ${color_g[2]}, ${color_g[3]})`
			} else if (ii === (color.length / 4 - 1) / 2) {
				// 光标中心点
				point_color_span_list[ii].id = 'point_color_t_c'
				point_color_span_list[ii].style.background = `rgba(${color_g[0]}, ${color_g[1]}, ${color_g[2]}, ${color_g[3]})`
				// 改变颜色文字和样式
				try {
					const color = (Color as {
						rgb: (color: any) => { hex: (x: any) => string, isLight: () => boolean }
					}).rgb(color_g)
					const the_text_color = [color.hex(undefined), color.isLight() ? '#000' : '#FFF']
					clip_copy.style.backgroundColor = the_text_color[0]
					const main_el = clip_copy
					// 只改变默认格式的字体颜色和内容，并定位展示
					main_el.style.color = the_text_color[1]
					main_el.innerText = Color(color_g).hex(undefined)
				} catch {
				}
			} else {
				point_color_span_list[ii].id = 'point_color_t_t'
				point_color_span_list[ii].style.background = `rgba(${color_g[0]}, ${color_g[1]}, ${color_g[2]}, ${color_g[3]})`
			}
		}
	}

	function final_rect_fix() {
		final_rect = final_rect.map((i) => Math.round(i))
		const x0 = final_rect[0]
		const y0 = final_rect[1]
		const x1 = final_rect[0] + final_rect[2]
		const y1 = final_rect[1] + final_rect[3]
		let x = Math.min(x0, x1)
		let y = Math.min(y0, y1)
		let w = Math.max(x0, x1) - x
		let h = Math.max(y0, y1) - y
		// 移出去移回来保持原来大小
		if (x < 0) w = x = 0
		if (y < 0) h = y = 0
		if (x > main_canvas.width) x = x % main_canvas.width
		if (y > main_canvas.height) y = y % main_canvas.height
		if (x + w > main_canvas.width) w = main_canvas.width - x
		if (y + h > main_canvas.height) h = main_canvas.height - y
		final_rect = [x, y, w, h]
	}

	// 判断光标位置并更改样式
	// 定义光标位置的移动方向
	function is_in_clip_rect(event) {
		const pos = p_xy_to_c_xy(event.offsetX, event.offsetY, event.offsetX, event.offsetY)
		const x = pos[0],
			y = pos[1],
			x0 = final_rect[0],
			x1 = final_rect[0] + final_rect[2],
			y0 = final_rect[1],
			y1 = final_rect[1] + final_rect[3]
		// 如果全屏,那允许框选
		if (!(final_rect[2] === main_canvas.width && final_rect[3] === main_canvas.height)) {
			in_rect = x0 <= x && x <= x1 && y0 <= y && y <= y1
			direction = ''
			const num = 8
			// 光标样式
			switch (true) {
				case x0 <= x && x <= x0 + num && y0 <= y && y <= y0 + num:
					clip_canvas.style.cursor = 'nwse-resize'
					direction = '西北'
					break
				case x1 - num <= x && x <= x1 && y1 - num <= y && y <= y1:
					clip_canvas.style.cursor = 'nwse-resize'
					direction = '东南'
					break
				case y0 <= y && y <= y0 + num && x1 - num <= x && x <= x1:
					clip_canvas.style.cursor = 'nesw-resize'
					direction = '东北'
					break
				case y1 - num <= y && y <= y1 && x0 <= x && x <= x0 + num:
					clip_canvas.style.cursor = 'nesw-resize'
					direction = '西南'
					break
				case x0 <= x && x <= x0 + num:
					clip_canvas.style.cursor = 'ew-resize'
					direction = '西'
					break
				case x1 - num <= x && x <= x1:
					clip_canvas.style.cursor = 'ew-resize'
					direction = '东'
					break
				case y0 <= y && y <= y0 + num:
					clip_canvas.style.cursor = 'ns-resize'
					direction = '北'
					break
				case y1 - num <= y && y <= y1:
					clip_canvas.style.cursor = 'ns-resize'
					direction = '南'
					break
				case x0 + num < x && x < x1 - num && y0 + num < y && y < y1 - num:
					clip_canvas.style.cursor = 'move'
					direction = 'move'
					break
				default:
					clip_canvas.style.cursor = 'crosshair'
					direction = ''
					break
			}
		} else {
			// 全屏可框选
			clip_canvas.style.cursor = 'move'
			direction = ''
			in_rect = false
		}
	}

	// 调整框选
	function move_rect(o_final_rect, oe, e) {
		const op = p_xy_to_c_xy(oe.offsetX, oe.offsetY, oe.offsetX, oe.offsetY)
		const p = p_xy_to_c_xy(e.offsetX, e.offsetY, e.offsetX, e.offsetY)
		const dx = p[0] - op[0]
		const dy = p[1] - op[1]
		switch (direction) {
			case '西北':
				final_rect = [o_final_rect[0] + dx, o_final_rect[1] + dy, o_final_rect[2] - dx, o_final_rect[3] - dy]
				break
			case '东南':
				final_rect = [o_final_rect[0], o_final_rect[1], o_final_rect[2] + dx, o_final_rect[3] + dy]
				break
			case '东北':
				final_rect = [o_final_rect[0], o_final_rect[1] + dy, o_final_rect[2] + dx, o_final_rect[3] - dy]
				break
			case '西南':
				final_rect = [o_final_rect[0] + dx, o_final_rect[1], o_final_rect[2] - dx, o_final_rect[3] + dy]
				break
			case '西':
				final_rect = [o_final_rect[0] + dx, o_final_rect[1], o_final_rect[2] - dx, o_final_rect[3]]
				break
			case '东':
				final_rect = [o_final_rect[0], o_final_rect[1], o_final_rect[2] + dx, o_final_rect[3]]
				break
			case '北':
				final_rect = [o_final_rect[0], o_final_rect[1] + dy, o_final_rect[2], o_final_rect[3] - dy]
				break
			case '南':
				final_rect = [o_final_rect[0], o_final_rect[1], o_final_rect[2], o_final_rect[3] + dy]
				break
			case 'move':
				final_rect = [o_final_rect[0] + dx, o_final_rect[1] + dy, o_final_rect[2], o_final_rect[3]]
				break
		}
		if (final_rect[0] < 0) {
			final_rect[2] = final_rect[2] + final_rect[0]
			final_rect[0] = 0
		}
		if (final_rect[1] < 0) {
			final_rect[3] = final_rect[3] + final_rect[1]
			final_rect[1] = 0
		}
		if (final_rect[0] + final_rect[2] > main_canvas.width) final_rect[2] = main_canvas.width - final_rect[0]
		if (final_rect[1] + final_rect[3] > main_canvas.height) final_rect[3] = main_canvas.height - final_rect[1]

		final_rect_fix()
		draw_clip_rect()
	}

	// 鼠标框选坐标转画布坐标,鼠标坐标转画布坐标
	function p_xy_to_c_xy(o_x1, o_y1, o_x2, o_y2) {
		// 0_零_1_一_2_二_3 阿拉伯数字为点坐标（canvas），汉字为像素坐标（html）
		// 输入为边框像素坐标
		// 为了让canvas获取全屏，边框像素点要包括
		// 像素坐标转为点坐标后,左和上(小的)是不变的,大的少1
		let x1 = Math.min(o_x1, o_x2)
		let y1 = Math.min(o_y1, o_y2)
		let x2 = Math.max(o_x1, o_x2) + 1
		let y2 = Math.max(o_y1, o_y2) + 1
		// canvas缩放变换
		x1 = Math.round(clip_canvas.width * (x1 / clip_canvas.offsetWidth))
		y1 = Math.round(clip_canvas.height * (y1 / clip_canvas.offsetHeight))
		x2 = Math.round(clip_canvas.width * (x2 / clip_canvas.offsetWidth))
		y2 = Math.round(clip_canvas.height * (y2 / clip_canvas.offsetHeight))
		return [x1, y1, x2 - x1, y2 - y1]
	}

	function get_clip_photo() {
		const main_ctx = main_canvas.getContext('2d')
		if (!final_rect)
			final_rect = [0, 0, main_canvas.width, main_canvas.height]

		const tmp_canvas = document.createElement('canvas')
		tmp_canvas.width = final_rect[2]
		tmp_canvas.height = final_rect[3]
		const gid = main_ctx.getImageData(final_rect[0], final_rect[1], final_rect[2], final_rect[3])
		// 裁剪
		tmp_canvas.getContext('2d').putImageData(gid, 0, 0)

		return tmp_canvas.toDataURL().replace(/^data:image\/\w+;base64,/, '')
	}
})
</script>

<style>
:root {
	--bar-bg: rgba(255, 255, 255, 0.4);
	--transition: 0.4s cubic-bezier(0.25, 1, 0.5, 1);
	--blur: blur(25px);
	--shadow: #0003 0 0 4px;
	--color: #000;
	--color-size: 150px;
	--color-i-size: 10px;
	--color-i-i: 15;
}

::-webkit-scrollbar {
	display: none;
}

html {
	cursor: none;
}

body {
	cursor: default;
	user-select: none;
}

html, body {
	margin: 0;
	width: 100%;
	height: 100%;
	color: var(--color);
}

#main_photo, #clip_photo {
	position: absolute;
}

#main_photo {
	z-index: 8;
}

#clip_photo {
	z-index: 10;
	cursor: crosshair;
}

#mouse_bar {
	display: flex;
	flex-direction: column;
	align-items: center;
	position: fixed;
	z-index: 100;
	border-radius: 8px;
	box-shadow: var(--shadow);
	backdrop-filter: var(--blur);
	background: var(--bar-bg);
	overflow: hidden;
	pointer-events: none;
}

#point_color {
	width: var(--color-size);
	height: var(--color-size);
	overflow: hidden;
	transition: var(--transition);
	display: grid;
	grid-template-columns: repeat(var(--color-i-i), var(--color-i-size));
	grid-template-rows: repeat(var(--color-i-i), var(--color-i-size));
}

#point_color_t_c {
	box-shadow: #000 0 0 0 1px, #FFF 0 0 0 2px;
	position: relative;
	z-index: 2;
}

/*noinspection CssUnusedSymbol*/
#point_color_t_b {
	box-shadow: #0005 0 0 0 0.25px inset, #FFF5 0 0 0 0.5px inset;
	opacity: 0.5;
}

#clip_copy {
	width: 100%;
	transition: var(--transition);
	overflow: hidden;
	height: 32px;
	line-height: 32px;
	text-align: center;
	white-space: nowrap;
}

#clip_wh {
	position: absolute;
	z-index: 11;
	backdrop-filter: var(--blur);
	background: var(--bar-bg);
	border-radius: 8px;
	box-shadow: var(--shadow);
	display: flex;
}

#clip_wh > div:focus {
	outline: none;
}

#clip_wh > div {
	cursor: text;
	margin: 8px;
}
</style>
