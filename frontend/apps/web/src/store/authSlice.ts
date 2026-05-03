import { createSlice, type PayloadAction } from "@reduxjs/toolkit"
import type { UserInfo } from "@/models/userModel"

interface AuthState {
  isLogin: boolean
  token: string | null
  user: UserInfo | null
}

const initialState: AuthState = {
  isLogin: false,
  token: null,
  user: null,
}

const authSlice = createSlice({
  name: "auth",
  initialState,
  reducers: {
    loginSuccess(
      state,
      action: PayloadAction<{ token: string; user: UserInfo }>
    ) {
      state.isLogin = true
      state.token = action.payload.token
      state.user = action.payload.user
    },
    logout(state) {
      state.isLogin = false
      state.token = null
      state.user = null
    },
    updateUser(state, action: PayloadAction<Partial<UserInfo>>) {
      if (!state.user) return
      state.user = {
        ...state.user,
        ...action.payload,
      }
    },
  },
})

export const { loginSuccess, logout, updateUser } = authSlice.actions

export default authSlice.reducer
