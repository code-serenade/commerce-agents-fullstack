import { configureStore } from "@reduxjs/toolkit"
import authReducer from "./authSlice"
import accessReducer from "./accessSlice"
import requestErrorReducer from "./requestErrorSlice"

export const store = configureStore({
  reducer: {
    auth: authReducer,
    adminAccess: accessReducer,
    requestError: requestErrorReducer,
  },
})

export type RootState = ReturnType<typeof store.getState>
export type AppDispatch = typeof store.dispatch
