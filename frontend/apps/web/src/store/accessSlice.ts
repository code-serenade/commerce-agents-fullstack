import { createSlice, type PayloadAction } from "@reduxjs/toolkit"

export interface WebAccessState {
  roleCodes: string[]
  permissionCodes: string[]
}

const initialState: WebAccessState = {
  roleCodes: [],
  permissionCodes: [],
}

const accessSlice = createSlice({
  name: "webAccess",
  initialState,
  reducers: {
    setAccess(state, action: PayloadAction<WebAccessState>) {
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
