const common = {
  actions: {
    cancel: "Cancel",
    close: "Close",
    delete: "Delete",
    rename: "Rename",
    save: "Save",
  },
  validation: {
    required: "This field is required",
    emailInvalid: "Please enter a valid email address",
    maxLength: "Use at most {{max}} characters",
  },
} as const

export default common
