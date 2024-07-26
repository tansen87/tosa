import { IBaseTransService } from '../../types'

export * from './Language'

import { Bing } from './Bing'

export const plugins: IBaseTransService[] = [Bing]
