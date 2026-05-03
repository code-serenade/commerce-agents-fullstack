import { createSlice, type PayloadAction } from "@reduxjs/toolkit"

interface RequestErrorState {
  message: string | null
}

const initialState: RequestErrorState = {
  message: null,
}

const requestErrorSlice = createSlice({
  name: "requestError",
  initialState,
  reducers: {
    setRequestErrorMessage(state, action: PayloadAction<string>) {
      state.message = action.payload
    },
    clearRequestErrorMessage(state) {
      state.message = null
    },
  },
})

export const { clearRequestErrorMessage, setRequestErrorMessage } =
  requestErrorSlice.actions

export default requestErrorSlice.reducer
