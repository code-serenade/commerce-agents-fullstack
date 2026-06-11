const common = {
  actions: {
    cancel: "取消",
    close: "关闭",
    delete: "删除",
    rename: "重命名",
    save: "保存",
  },
  validation: {
    required: "此项必填",
    emailInvalid: "请输入有效邮箱地址",
    maxLength: "最多 {{max}} 个字符",
  },
} as const

export default common
