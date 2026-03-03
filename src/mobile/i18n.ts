import i18n from "i18next";
import { initReactI18next } from "react-i18next";

const resources = {
  en: {
    translation: {
      status: {
        connected: "Connected",
        disconnected: "Disconnected",
        connecting: "Connecting...",
        reconnecting: "Reconnecting...",
        connectionLost: "Connection lost. Refresh to try again.",
      },
      input: {
        placeholder: "Type text to send to your PC...",
        restore: "Restore Last Message",
        charCount: "{{count}} / 10000",
        textTooLong: "Text too long (max 10000 characters)",
      },
      actions: {
        send: "Send to PC",
        sending: "Sending...",
      },
      notifications: {
        sent: "Text sent successfully!",
        lastMessageRestored: "Last message restored",
        sendFailed: "Failed to send text",
      },
      devices: {
        add: "Add",
        emptyState: "No devices yet. Tap Add to connect.",
        addDevice: "Add Device",
        scanQR: "Scan QR",
        inputUrl: "Enter URL",
        startScan: "Start Scan",
        stopScan: "Stop Scan",
        scanHint: "Tap Start Scan and scan the QR on your PC.",
        connectionAddress: "Connection Address",
        deviceNameOptional: "Device name (optional)",
        urlPlaceholder: "http://192.168.1.100:38425",
        willConnectTo: "Will connect to: {{url}}",
        namePlaceholder: "Default: PC-xxx",
        renameDevice: "Rename Device",
        cancel: "Cancel",
        save: "Save",
        selectDevice: "Please select a device first",
        deviceNotConnected: "Device not connected",
        invalidUrl: "Invalid address",
        sendFailed: "Send failed, please check connection",
        sendTimeout: "Send timed out, please retry",
        maxLimit: "Maximum 5 devices supported",
        alreadyExists: "Device already exists",
      },
      settings: {
        title: "Settings",
        language: "Language",
        theme: "Theme",
        themeSystem: "System",
        themeLight: "Light",
        themeDark: "Dark",
      },
    },
  },
  zh: {
    translation: {
      status: {
        connected: "已连接",
        disconnected: "未连接",
        connecting: "连接中...",
        reconnecting: "重连中...",
        connectionLost: "连接已断开，请刷新页面重试",
      },
      input: {
        placeholder: "输入要发送到电脑的文字...",
        restore: "恢复上一条消息",
        charCount: "{{count}} / 10000",
        textTooLong: "文本过长（最多10000字符）",
      },
      actions: {
        send: "发送到电脑",
        sending: "发送中...",
      },
      notifications: {
        sent: "发送成功！",
        lastMessageRestored: "已恢复上一条消息",
        sendFailed: "发送失败",
      },
      devices: {
        add: "添加",
        emptyState: "暂无设备，请点击添加",
        addDevice: "添加设备",
        scanQR: "扫描二维码",
        inputUrl: "输入地址",
        startScan: "开始扫描",
        stopScan: "停止扫描",
        scanHint: "点击开始扫描，并扫描电脑端二维码",
        connectionAddress: "连接地址",
        deviceNameOptional: "设备名称（可选）",
        urlPlaceholder: "http://192.168.1.100:38425",
        willConnectTo: "将连接到: {{url}}",
        namePlaceholder: "默认: PC-xxx",
        renameDevice: "重命名设备",
        cancel: "取消",
        save: "保存",
        selectDevice: "请先选择设备",
        deviceNotConnected: "设备未连接",
        invalidUrl: "无效的连接地址",
        sendFailed: "发送失败，请检查连接",
        sendTimeout: "发送超时，请重试",
        maxLimit: "最多支持 5 台设备",
        alreadyExists: "此设备已存在",
      },
      settings: {
        title: "设置",
        language: "语言",
        theme: "主题",
        themeSystem: "跟随系统",
        themeLight: "浅色",
        themeDark: "深色",
      },
    },
  },
} as const;

function detectLanguage(): "en" | "zh" {
  const browserLang = navigator.language || (navigator as any).userLanguage;
  if (typeof browserLang === "string" && browserLang.toLowerCase().startsWith("zh")) {
    return "zh";
  }
  return "en";
}

function getInitialLanguage(): "en" | "zh" {
  const saved = localStorage.getItem("dropvoice-mobile-lang");
  if (saved === "en" || saved === "zh") return saved;
  return detectLanguage();
}

export async function initMobileI18n() {
  await i18n.use(initReactI18next).init({
    resources: resources as any,
    lng: getInitialLanguage(),
    fallbackLng: "en",
    interpolation: {
      escapeValue: false,
    },
    react: {
      useSuspense: false,
    },
  });
}

export function setMobileLanguage(lang: "en" | "zh") {
  localStorage.setItem("dropvoice-mobile-lang", lang);
  i18n.changeLanguage(lang);
}

export default i18n;
