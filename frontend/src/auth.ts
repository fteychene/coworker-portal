interface TokenPayload {
  sub: number
  username: string
  first_name: string
  exp: number
  iat: number
}

export function getTokenPayload(): TokenPayload | null {
  const token = localStorage.getItem('token')
  if (!token) return null
  try {
    return JSON.parse(atob(token.split('.')[1])) as TokenPayload
  } catch {
    return null
  }
}

export function isAuthenticated(): boolean {
  const payload = getTokenPayload()
  if (!payload) return false
  return payload.exp * 1000 > Date.now()
}

export function logout() {
  localStorage.removeItem('token')
}
