import type { FlattenKeys, TranslationBundle } from './types';

import common from '../locales/en/common.json';
import devices from '../locales/en/devices.json';
import errors from '../locales/en/errors.json';
import landing from '../locales/en/landing.json';
import settings from '../locales/en/settings.json';

const en: TranslationBundle = { common, devices, errors, landing, settings };

export default en;

export type TranslationKey = FlattenKeys<TranslationBundle>;
