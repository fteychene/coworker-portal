import { useState } from 'react'
import { Link } from 'react-router-dom'
import { getTokenPayload, isAuthenticated, logout } from '../auth'
import { type ThemeId, getTheme, setTheme } from '../lib/theme'

export function Navbar() {
  const user = getTokenPayload()
  const authenticated = isAuthenticated()
  const [theme, setThemeState] = useState<ThemeId>(getTheme)

  const handleThemeChange = (t: ThemeId) => {
    setTheme(t)
    setThemeState(t)
  }

  const handleLogout = () => {
    logout()
    window.location.href = '/login'
  }

  return (
    <div className="navbar bg-base-100 border-b border-base-200 px-4">
      <div className="navbar-start">
        <Link to={authenticated ? '/dashboard' : '/login'}>
          <img src="/logo-default.png" alt="Coworking" className="h-8" />
        </Link>
      </div>
      <div className="navbar-end gap-3">
        {user && (
          <span className="text-sm text-base-content/60 hidden sm:block">
            {user.first_name}
          </span>
        )}
        <label className="flex items-center gap-1 cursor-pointer" title={theme === 'corporate' ? 'Passer en mode sombre' : 'Passer en mode clair'}>
          <span className="text-sm">☀️</span>
          <input
            type="checkbox"
            className="toggle toggle-sm"
            checked={theme === 'dim'}
            onChange={e => handleThemeChange(e.target.checked ? 'dim' : 'corporate')}
            aria-label="Changer de thème"
          />
          <span className="text-sm">🌙</span>
        </label>
        {authenticated ? (
          <button className="btn btn-ghost btn-sm" onClick={handleLogout}>
            Se déconnecter
          </button>
        ) : (
          <Link to="/login" className="btn btn-ghost btn-sm">
            Se connecter
          </Link>
        )}
      </div>
    </div>
  )
}
