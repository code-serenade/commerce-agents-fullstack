import { createSlice, type PayloadAction } from "@reduxjs/toolkit"

export interface AdminAccessState {
  roleCodes: string[]
  permissionCodes: string[]
}

const initialState: AdminAccessState = {
  roleCodes: [],
  permissionCodes: [],
}

const accessSlice = createSlice({
  name: "adminAccess",
  initialState,
  reducers: {
    setAccess(state, action: PayloadAction<AdminAccessState>) {
      state.roleCodes = action.payload.roleCodes
      state.permissionCodes = action.payload.permissionCodes
    },
    resetAccess(state) {
      state.roleCodes = []
      state.permissionCodes = []
    },
  },
})

export const { resetAccess, setAccess } = accessSlice.actions

export default accessSlice.reducer
