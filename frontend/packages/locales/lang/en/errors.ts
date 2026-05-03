const error = {
  default: "unauthorized or unknown error, please try again later",
  "-1": "server internal error",
  "-10000": "username already exists",
  "-10001": "email already exists",
  "-10002": "user not found",
  "-10003": "user disabled",
  "-10004": "password hash failed",
  "-10005": "password hash parse failed",
  "-10006": "invalid password",
  "-11000": "admin user not found",
  "-12000": "app user not found",
  "-12001": "app user disabled",
} as const

export default error
